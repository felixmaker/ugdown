use std::{
    cell::Cell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use fltk_table::SmartTable;
use uuid::Uuid;

use crate::downloader::*;
use fltk::{prelude::*, *};

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

pub struct TaskRow {
    pub id: Uuid,
    pub title: String,
    pub extension: String,
    pub size: usize,
    pub percent: f64,
    pub eta: usize,
    pub speed: usize,
    pub status: TaskStatus,
    pub info: DownloadInfo,
}

impl TaskRow {
    pub fn update(&mut self, persent: f64, dur: f64) {
        if persent > self.percent {
            let speed = (persent - self.percent) / dur;
            self.speed = (speed * self.size as f64) as usize;
            self.eta = ((1.0 - persent) / speed) as usize;
            self.percent = persent;
        }
    }
}

pub struct TaskTable {
    table: SmartTable,
    pub rows: Arc<Mutex<HashMap<Uuid, TaskRow>>>,
    pub row_order: Arc<Mutex<VecDeque<Uuid>>>,
}

impl TaskTable {
    pub fn default() -> Self {
        let mut table = SmartTable::default_fill();
        set_table_opts(&mut table);

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

        let rows: Arc<Mutex<HashMap<Uuid, TaskRow>>> = Default::default();
        let row_order: Arc<Mutex<VecDeque<Uuid>>> = Default::default();

        Self {
            table,
            rows,
            row_order,
        }
    }

    pub fn add_download_info(&mut self, download_info: &[DownloadInfo]) {
        {
            let mut rows = self.rows.lock().unwrap();
            let mut row_order = self.row_order.lock().unwrap();

            for info in download_info {
                let id: Uuid = uuid::Uuid::new_v4();

                let row: TaskRow = TaskRow {
                    id,
                    extension: info.ext.clone(),
                    percent: 0 as f64,
                    title: info.title.clone(),
                    size: info.stream_size,
                    speed: 0,
                    status: TaskStatus::Queued,
                    eta: 60 * 60 * 24,
                    info: info.clone(),
                };

                rows.insert(id, row);
                row_order.push_back(id);
            }
        }

        self.update_rows()
    }

    pub fn remove_download_info(&mut self, id: &Vec<Uuid>) {
        {
            let mut rows = self.rows.lock().unwrap();
            let mut row_order = self.row_order.lock().unwrap();

            for id in id {
                rows.remove(&id);

                for i in 0..rows.len() {
                    if id == &row_order[i] {
                        row_order.remove(i);
                    }
                }
            }
        }
        self.table.clear();
        self.update_rows();
        self.table.unset_selection();
    }

    fn get_select_uuid(&self) -> Vec<Uuid> {
        let (row_top, _, row_bot, _) = self.table.get_selection();

        let row_order = self.row_order.lock().unwrap();
        let mut uuid_vec = Vec::new();
        for i in row_top..=row_bot {
            if let Some(id) = row_order.get(i as usize) {
                uuid_vec.push(id.clone());
            }
        }
        uuid_vec
    }

    pub fn remove_select(&mut self) {
        let uuid_vec = self.get_select_uuid();
        self.remove_download_info(&uuid_vec);
    }

    pub fn reload(&mut self) {
        self.table.clear();
        self.update_rows();
    }

    pub fn update_rows(&mut self) {
        let rows = self.rows.clone();
        let rows = rows.lock().unwrap();
        let row_order = self.row_order.clone();
        let row_order = row_order.lock().unwrap();

        let mut i = 0;
        for row_id in row_order.iter() {
            if let Some(row) = rows.get(row_id) {
                self.set_task_row(i, row);
                i = i + 1;
            }
        }

        self.table.redraw();
    }

    pub fn start_task(&self, uuid: Uuid) {
        let task_map = self.rows.clone();
        let is_running = {
            if let Some(task) = task_map.lock().unwrap().get_mut(&uuid) {
                match task.status {
                    TaskStatus::Running => true,
                    _ => false,
                }
            } else {
                false
            }
        };

        if is_running == false {
            std::thread::spawn({
                let task_map = self.rows.clone();
                {
                    if let Some(task) = task_map.lock().unwrap().get_mut(&uuid) {
                        task.status = TaskStatus::Running;
                    }
                }
                move || {
                    if let Some(info) = {
                        let task_map = task_map.lock().unwrap();
                        task_map.get(&uuid).and_then(|x| Some(x.info.to_owned()))
                    } {
                        let engine = info.downloader;
                        let download_id = info.stream_id;
                        let url = info.url;

                        let instant: Rc<Cell<Instant>> = Rc::new(Cell::new(Instant::now()));
                        let (output_dir, output_name) = info
                            .save_option
                            .and_then(|x| Some((x.output_dir, x.file_name)))
                            .unwrap_or(("./".to_owned(), format!("{}.{}", info.title, info.ext)));

                        if let Err(error) = download_by_id(
                            &engine,
                            &url,
                            &download_id,
                            &output_dir,
                            &output_name,
                            {
                                let task_map = task_map.clone();
                                let instant = instant.clone();
                                move |persent| {
                                    let before = instant.get();
                                    let now = Instant::now();
                                    instant.set(now);
                                    let dur = (now - before).as_secs_f64();
                                    if let Some(row) = task_map.lock().unwrap().get_mut(&uuid) {
                                        row.update(persent, dur);
                                    }
                                }
                            },
                        ) {
                            println!("{}", error);
                        }

                        {
                            if let Some(task) = task_map.lock().unwrap().get_mut(&uuid) {
                                task.status = TaskStatus::Stopped;
                            }
                        }
                    }
                }
            });
        }
    }

    pub fn start_select(&self) {
        let uuid_vec = self.get_select_uuid();
        for uuid in uuid_vec {
            self.start_task(uuid);
        }
    }

    fn set_task_row(&mut self, row: i32, task_row: &TaskRow) {
        self.table.set_cell_value(row, 0, &task_row.title);
        self.table.set_cell_value(row, 1, &task_row.extension);
        self.table
            .set_cell_value(row, 2, &size_to_string(task_row.size));
        self.table
            .set_cell_value(row, 3, &percent_to_string(task_row.percent));
        self.table
            .set_cell_value(row, 4, &eta_to_string(task_row.eta));
        self.table
            .set_cell_value(row, 5, &speed_to_string(task_row.speed));
        self.table
            .set_cell_value(row, 6, &task_row.status.to_string());
    }
}

widget_extends!(TaskTable, SmartTable, table);

pub fn size_to_string(size: usize) -> String {
    match size {
        gb if gb >= 1000 * 1000 * 1000 => format!("{:.2} GB", gb as f64 / 1000000000.0),
        mb if mb >= 1000 * 1000 => format!("{:.2} MB", mb as f64 / 1000000.0),
        kb if kb >= 1000 => format!("{:.2} KB", kb as f64 / 1000.0),
        b => format!("{} B", b),
    }
}

fn speed_to_string(size: usize) -> String {
    let speed = size_to_string(size);
    format!("{}/s", speed)
}

fn percent_to_string(percent: f64) -> String {
    format!("{:.1}%", percent * 100.0)
}

fn eta_to_string(eta: usize) -> String {
    let days = eta / (60 * 60 * 24);
    let hours = (eta - days * 60 * 60 * 24) / (60 * 60);
    let minutes = (eta - days * 60 * 60 * 24 - hours * 60 * 60) / 60;
    let seconds = eta - days * 60 * 60 * 24 - hours * 60 * 60 - minutes * 60;

    match (days, hours, minutes, seconds) {
        (d, _, _, _) if d != 0 => format!(">= {}d", d),
        (_, h, _, _) if h != 0 => format!(">= {}h", h),
        (_, _, m, s) if m != 0 => format!("{}m {}s", m, s),
        (_, _, _, s) => format!("{}s", s),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_size_to_string() {
        assert_eq!("1.20 MB", size_to_string(1200000));
        assert_eq!("1.22 GB", size_to_string(1220000170));
        assert_eq!("120 B", size_to_string(120));
    }

    #[test]
    fn test_percent_to_string() {
        assert_eq!("22.4%", percent_to_string(0.2242));
    }

    #[test]
    fn test_eta_to_string() {
        assert_eq!("1m 40s", eta_to_string(100));
        assert_eq!(">= 2h", eta_to_string(2 * 60 * 60 + 500));
        assert_eq!(">= 1d", eta_to_string(24 * 60 * 60 + 666));
    }
}
