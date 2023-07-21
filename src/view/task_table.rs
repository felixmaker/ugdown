use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader},
    rc::Rc,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    time::Instant,
};

use fltk_table::SmartTable;
use regex::Regex;
use uuid::Uuid;

use super::utils::*;
use crate::downloader::*;
use fltk::{prelude::*, *};

use anyhow::Result;

#[derive(Clone)]
pub struct TaskTable {
    table: SmartTable,
    task_queue: TaskQueue,
}

impl TaskTable {
    pub fn default() -> Self {
        let mut table = SmartTable::default_fill();
        Self::set_table_opts(&mut table);

        table.handle(|tb, event| {
            match event {
                enums::Event::Released => {
                    let (row_top, _, row_button, _) = tb.get_selection();
                    tb.set_selection(row_top, 0, row_button, 6);
                }
                _ => {}
            }
            true
        });

        let task_queue = TaskQueue::default();

        Self { table, task_queue }
    }

    pub fn update_rows(&mut self) {
        let mut i = 0;
        for uuid in self.task_queue.order.clone().borrow().iter() {
            if let Ok(task) = self.task_queue.get_task(*uuid) {
                let task = task.lock().unwrap();
                self.set_task_row(
                    i,
                    &task.download_info.title,
                    &task.download_info.ext,
                    task.download_info.stream_size,
                    task.task_info.progress,
                    task.task_info.eta,
                    (task.task_info.speed * task.download_info.stream_size as f64) as usize,
                    task.task_status,
                );
                i = i + 1;
            }
        }

        self.table.redraw();
    }

    pub fn add_tasks(&mut self, download_info_vec: &Vec<DownloadInfo>) {
        for download_info in download_info_vec {
            self.task_queue.add_task(download_info);
        }
        self.update_rows()
    }

    pub fn remove_tasks(&mut self, uuid_vec: &Vec<Uuid>) -> Result<()> {
        for uuid in uuid_vec {
            self.task_queue.remove_task(*uuid)?;
        }
        self.table.clear();
        self.update_rows();
        self.table.unset_selection();

        Ok(())
    }

    fn get_select_uuid(&self) -> Vec<Uuid> {
        let (row_top, _, row_bot, _) = self.table.get_selection();

        let mut uuid_vec = Vec::new();
        for i in row_top..=row_bot {
            if let Some(id) = self.task_queue.order.borrow().get(i as usize) {
                uuid_vec.push(id.clone());
            }
        }
        uuid_vec
    }

    pub fn remove_select(&mut self) -> Result<usize> {
        let uuid_vec = self.get_select_uuid();
        let length = uuid_vec.len();
        self.remove_tasks(&uuid_vec)?;
        Ok(length)
    }

    pub fn reload(&mut self) {
        self.table.clear();
        self.update_rows();
    }

    pub fn start_select(&mut self) -> Result<usize> {
        let uuid_vec = self.get_select_uuid();
        let length = uuid_vec.len();
        for uuid in uuid_vec {
            self.task_queue.start_task(uuid)?;
        }
        self.update_rows();

        Ok(length)
    }

    pub fn stop_select(&mut self) -> Result<usize> {
        let uuid_vec = self.get_select_uuid();
        let length = uuid_vec.len();
        for uuid in uuid_vec {
            self.task_queue.kill_task(uuid)?;
        }
        self.update_rows();
        Ok(length)
    }

    fn set_task_row(
        &mut self,
        row: i32,
        title: &str,
        extension: &str,
        size: usize,
        percent: f64,
        eta: usize,
        speed: usize,
        status: TaskStatus,
    ) {
        self.table.set_cell_value(row, 0, title);
        self.table.set_cell_value(row, 1, extension);
        self.table.set_cell_value(row, 2, &size_to_string(size));
        self.table
            .set_cell_value(row, 3, &percent_to_string(percent));
        self.table.set_cell_value(row, 4, &eta_to_string(eta));
        self.table.set_cell_value(row, 5, &speed_to_string(speed));
        self.table.set_cell_value(row, 6, &status.to_string());
    }

    fn set_table_opts(table: &mut SmartTable) {
        table.set_opts(fltk_table::TableOpts {
            rows: 30,
            cols: 7,
            editable: false,
            cell_border_color: enums::Color::from_rgb(255, 255, 255),
            cell_align: enums::Align::Left,
            ..Default::default()
        });
        table.set_row_height_all(20);
        table.set_col_width(0, 220);
        table.set_col_header_value(0, "Title");
        table.set_col_header_value(1, "Extension");
        table.set_col_header_value(2, "Size");
        table.set_col_header_value(3, "Percent");
        table.set_col_header_value(4, "ETA");
        table.set_col_header_value(5, "Speed");
        table.set_col_header_value(6, "Status");
    }
}

widget_extends!(TaskTable, SmartTable, table);

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TaskStatus {
    Queued,
    Running,
    Stopped,
}

impl TaskStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Queued => "Queued".to_string(),
            Self::Running => "Running".to_string(),
            Self::Stopped => "Stopped".to_string(),
        }
    }
}

struct Task {
    _uuid: Uuid,
    download_info: DownloadInfo,
    task_status: TaskStatus,
    task_killer: Option<Sender<bool>>,
    task_info: TaskInfo,
}

struct TaskInfo {
    progress: f64,
    speed: f64,
    eta: usize,
}

impl TaskInfo {
    pub fn default() -> Self {
        Self {
            progress: 0.0,
            speed: 0.0,
            eta: 60 * 60 * 24,
        }
    }

    pub fn update(&mut self, progress: f64, dur: f64) {
        if progress > self.progress {
            self.speed = (progress - self.progress) / dur;
            self.eta = ((1.0 - progress) / self.speed) as usize;
            self.progress = progress;
        }
    }
}

#[derive(Clone, Default)]
struct TaskQueue {
    inner: Rc<RefCell<HashMap<Uuid, Arc<Mutex<Task>>>>>,
    order: Rc<RefCell<VecDeque<Uuid>>>,
}

impl TaskQueue {
    fn start_task(&self, uuid: Uuid) -> Result<()> {
        let task = self.get_task(uuid)?;

        if TaskStatus::Running == {
            let task = task.lock().unwrap();
            task.task_status
        } {
            return Ok(());
        }

        let (sender, receiver) = mpsc::channel::<bool>();
        {
            let mut task = task.lock().unwrap();
            task.task_killer = Some(sender.clone());
        }

        std::thread::spawn({
            move || {
                match {
                    let mut task = task.lock().unwrap();
                    task.task_status = TaskStatus::Running;
                    execute_download_info(&task.download_info)
                } {
                    Ok((mut child, cookie_file, read_stderr)) => {
                        let mut reader: Box<dyn BufRead> = {
                            if read_stderr {
                                Box::new(BufReader::new(child.stderr.take().unwrap()))
                            } else {
                                Box::new(BufReader::new(child.stdout.take().unwrap()))
                            }
                        };

                        let mut buf = Vec::new();
                        let mut before = Instant::now();
                        let re: Regex = Regex::new(r"(?<progress>[0-9\.]*?)%").unwrap();
                        while let Ok(length) = reader.read_until(b'%', &mut buf) {
                            if let Ok(should_kill) = receiver.try_recv() {
                                if should_kill {
                                    let _ = child.kill();
                                    let mut task = task.lock().unwrap();
                                    task.task_status = TaskStatus::Stopped;
                                    break;
                                }
                            }
                            match length {
                                0 => break,
                                _ => {
                                    let result = String::from_utf8_lossy(&buf);
                                    let result = result.trim();
                                    if let Some(caps) = re.captures(result) {
                                        let progress = caps.name("progress").unwrap();
                                        let progress =
                                            progress.as_str().parse::<f64>().unwrap_or(-1.0)
                                                / 100.0;

                                        let now = Instant::now();
                                        let dur = now - before;
                                        before = now;

                                        let mut task = task.lock().unwrap();
                                        task.task_info.update(progress, dur.as_secs_f64());
                                    }
                                    buf.clear();
                                }
                            }
                        }

                        if let Some(cookie_file) = cookie_file {
                            let _ = std::fs::remove_file(cookie_file);
                        }
                    }
                    Err(error) => {
                        println!("{}", error)
                    }
                }

                task.lock().unwrap().task_status = TaskStatus::Stopped;
            }
        });

        Ok(())
    }

    fn kill_task(&self, uuid: Uuid) -> Result<()> {
        let task = self.get_task(uuid)?;
        let mut task = task.lock().unwrap();

        if let Some(sender) = task.task_killer.take() {
            sender.send(true)?;
        }

        Ok(())
    }

    fn add_task(&self, download_info: &DownloadInfo) {
        let uuid = Uuid::new_v4();

        let task = Task {
            _uuid: uuid,
            download_info: download_info.to_owned(),
            task_status: TaskStatus::Queued,
            task_killer: None,
            task_info: TaskInfo::default(),
        };

        self.inner
            .borrow_mut()
            .insert(uuid, Arc::new(Mutex::new(task)));
        self.order.borrow_mut().push_back(uuid);
    }

    fn remove_task(&self, uuid: Uuid) -> Result<()> {
        self.kill_task(uuid)?;
        self.inner.borrow_mut().remove(&uuid);
        let order = self.order.borrow().clone();
        for i in 0..order.len() {
            if order[i] == uuid {
                self.order.borrow_mut().remove(i);
            }
        }
        Ok(())
    }

    fn get_task(&self, uuid: Uuid) -> Result<Arc<Mutex<Task>>> {
        let task = self
            .inner
            .borrow()
            .get(&uuid)
            .ok_or_else(|| anyhow::anyhow!("No such task"))?
            .to_owned();
        Ok(task)
    }
}
