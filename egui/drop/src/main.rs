#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

// 入力中のファイルパスと現在の補完名をとり、次の補完名候補を返す
// 補完名候補がない場合は入力値を返す
fn complete_next(input_path: &str, current_completion: &str) -> String {
    let mut completion = std::path::PathBuf::from(current_completion);
    let path = std::path::PathBuf::from(input_path);

    let dir_path: std::path::PathBuf;
    let start_name: String;
    if path.is_dir() {
	dir_path = path.clone();
	start_name = "".to_string();
    } else {
	dir_path = path.parent().unwrap_or(std::path::Path::new("")).to_path_buf();
	start_name = path.file_name().unwrap_or(std::ffi::OsStr::new("")).to_string_lossy().to_string();
    }
    // ディレクトリでない場合は、何もしない
    if dir_path.is_dir() {
    
	// ディレクトリの中身を取得
	if let Ok(files) = dir_path.read_dir() {
	    // そのディレクトリ内のstart_nameで始まるファイル名を取得する
	    let mut candidates = files.filter_map(|e| {
		let path = e.unwrap().path();
		let name = path.file_name().unwrap().to_string_lossy().to_string();
		if name.starts_with(&start_name) {
		    Some(path)
		} else {
		    None
		}}).collect::<Vec<_>>();
	    // ソートしておく
	    candidates.sort();
	    // 次の補完名候補を取得する
	    if let Some(index) = candidates.iter().position(|s| s == completion.as_path()) {
		if index < candidates.len() - 1 {
		    completion = candidates[index + 1].clone();
		} else {
		    completion = candidates[0].clone();
		}
	    } else {
		completion = candidates[0].clone();
	    }
	}
    }
    // ディレクトリ名を付加する
    return completion.to_string_lossy().to_string();
}

#[derive(Default)]
struct FileSelectorFrame {
    picked: bool,
    path: String,
    input: String,
}

impl FileSelectorFrame {
    fn add_widgets(&mut self, ui: &mut egui::Ui) -> Option<String>{
	// 右端にボタンを配置する
	ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
	    if ui.button("Open file…").clicked() {
		if let Some(path) = rfd::FileDialog::new().pick_file() {
		    self.path = path.display().to_string();
		    self.picked = true;
		}
	    }
	    // 残りは全て１行テキスト
	    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true), |ui| {
		// tabで移動しないsingleline text edit を作成
		let response = ui.add(egui::TextEdit::singleline(&mut self.path).lock_focus(true));
		// enter押されたら、ファイルが選択されたとみなす
		if response.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
		    self.picked = true;
		// tab押されたら、補完する
		} else if response.ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
		    if self.input.is_empty() {
			self.input = self.path.clone();
		    }
		    self.path = complete_next(&self.input, &self.path);
		// 上記以外のキー入力あれば入力を一旦クリア
		} else {
		    if response.changed() {
			self.input.clear();
		    }
		}
	    });
	});
	if self.picked {
	    self.picked = false;
	    Some(self.path.clone())
	} else {
	    None
	}
    }

    fn set_picked_path(&mut self, path: String) {
	self.path = path;
	self.picked = true;
    }
}

#[derive(Default)]
struct MyApp {
    fileselector: FileSelectorFrame,
    contents: String,
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

	// eguiでは今のところウィジットへのファイルのドラッグアンドドロップはサポートされていないみたい。
	// ドロップされたファイルを取得するためには、eframe::Frameのinputメソッドを使う必要がある。
	// ドロップ処理は fileselector に隠蔽したいが良い方法が浮かばないので、ここでドロップされたファイルを取得して、fileselectorに渡す。
	// Collect dropped files:
        ctx.input(|i| {
	    if !i.raw.dropped_files.is_empty() {
		self.fileselector.set_picked_path(i.raw.dropped_files[0].path.as_ref().unwrap().display().to_string());
	    }
        });

        egui::CentralPanel::default().show(ctx, |ui| {

	    if let Some(path) = self.fileselector.add_widgets(ui) {
		// If the file exists at path, read the contents
		if let Ok(contents) = std::fs::read_to_string(&path) {
		    self.contents = contents;
		}
	    }
	    
	    egui::ScrollArea::vertical().show(ui, |ui| {
		ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_justify(true), |ui| {
		    ui.text_edit_multiline(&mut self.contents);
		});
            });
        });
    }
}

    
