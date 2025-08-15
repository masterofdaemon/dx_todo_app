use dioxus::prelude::*;
use dioxus_router::prelude::*;
use dioxus::events::Key;
use std::cmp::max;
use printpdf::{PdfDocument, PdfDocumentReference, Mm, BuiltinFont, IndirectFontRef};
use printpdf::indices::{PdfPageIndex, PdfLayerIndex};
use dioxus_i18n::{prelude::*, t};
use dioxus_i18n::prelude::Locale;
use unic_langid::langid;
#[cfg(all(not(target_os = "android"), feature = "desktop"))]
use rfd::FileDialog;

use crate::models::{Filter, Todo, Subtask, Project};
use crate::storage::{load_or_migrate_projects, save_projects};
use crate::components::{header::Header, add_form::AddForm, filter_bar::FilterBar};
use crate::components::projects::ProjectsState;
use crate::components::header::HeaderState;

// Use relative asset paths on Android (no leading slash), absolute on others
#[cfg(target_os = "android")]
pub const FAVICON: Asset = asset!("assets/favicon.ico");
#[cfg(not(target_os = "android"))]
pub const FAVICON: Asset = asset!("/assets/favicon.ico");

#[cfg(target_os = "android")]
pub const MAIN_CSS: Asset = asset!("assets/main.css");
#[cfg(not(target_os = "android"))]
pub const MAIN_CSS: Asset = asset!("/assets/main.css");

// Head nodes helper: on Android, skip document::* tags; on others, include meta, favicon, and stylesheet
#[cfg(target_os = "android")]
fn head_nodes() -> Element {
    const INLINE_CSS: &str = include_str!("../assets/main.css");
    rsx! { style { "{INLINE_CSS}" } }
}

#[cfg(not(target_os = "android"))]
fn head_nodes() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
    }
}

// Removed Android inline style helper to avoid rsx macro issues

// Disable Android JNI toast; we use an in-app flash instead
#[cfg(any(target_os = "android", not(target_os = "android")))]
fn android_toast(_msg: &str) {}

// Android: export without a file dialog – write into app external Downloads directory
#[cfg(target_os = "android")]
fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    // Create doc and fonts
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Project Report — {}", project.name),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("font error: {e}"))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("font error: {e}"))?;

    // Layout
    let mut current_page: PdfPageIndex = page1;
    let mut current_layer_idx: PdfLayerIndex = layer1;
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let line_height = Mm(6.0);
    let mut cursor_y = Mm(297.0) - margin_top;

    // Helper to write one line, handling pagination, using indices to avoid borrow issues
    fn write_line(
        doc: &PdfDocumentReference,
        current_page: &mut PdfPageIndex,
        current_layer_idx: &mut PdfLayerIndex,
        cursor_y: &mut Mm,
        margin_left: Mm,
        margin_top: Mm,
        line_height: Mm,
        font: &IndirectFontRef,
        text: &str,
        size_pt: f64,
    ) {
        if cursor_y.0 < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            *current_page = p;
            *current_layer_idx = l;
            *cursor_y = Mm(297.0) - margin_top;
        }
        let layer = doc.get_page(*current_page).get_layer(*current_layer_idx);
        layer.use_text(text, size_pt, margin_left, *cursor_y, font);
        *cursor_y = Mm(cursor_y.0 - line_height.0);
    }
    fn wrap_text(s: &str, max_chars: usize) -> Vec<String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for w in words {
            if line.is_empty() {
                line.push_str(w);
            } else if line.len() + 1 + w.len() <= max_chars {
                line.push(' ');
                line.push_str(w);
            } else {
                lines.push(line);
                line = w.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        if lines.is_empty() { vec![String::new()] } else { lines }
    }

    // Data & Summary
    let total_tasks = project.todos.len();
    let completed_tasks = project.todos.iter().filter(|t| t.completed).count();
    let active_tasks = total_tasks - completed_tasks;
    let total_subtasks: usize = project.todos.iter().map(|t| t.subtasks.len()).sum();
    let completion = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Header
    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_bold, "Project Report", 18.0);
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &project.name, 15.0);
    let generated_line = format!("Generated: {}", date_str);
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &generated_line, 10.0);
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, "", 8.0);

    // Summary block
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_bold, "Summary", 14.0);
    let summary = format!(
        "Tasks: {}  •  Completed: {}  •  Active: {}  •  Subtasks: {}  •  Completion: {:.1}%",
        total_tasks, completed_tasks, active_tasks, total_subtasks, completion
    );
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &summary, 11.0);
    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, "", 6.0);

    // Section: Active Tasks
    if active_tasks > 0 {
        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_bold, "Active Tasks", 13.0);
        for t in project.todos.iter().filter(|t| !t.completed) {
            let first = format!("[ ] {}", t.title);
            for (i, line) in wrap_text(&first, 90).into_iter().enumerate() {
                if i == 0 {
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &line, 12.0);
                } else {
                    let ind = format!("    {}", line);
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 12.0);
                }
            }
            if !t.description.trim().is_empty() {
                let desc = format!("— {}", t.description.trim());
                for line in wrap_text(&desc, 95) {
                    let ind = format!("    {}", line);
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 10.0);
                }
            }
            for s in &t.subtasks {
                let sub = format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title);
                for (i, line) in wrap_text(&sub, 95).into_iter().enumerate() {
                    if i == 0 {
                        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &line, 11.0);
                    } else {
                        let ind = format!("    {}", line);
                        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 11.0);
                    }
                }
            }
        }
        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, "", 6.0);
    }

    // Section: Completed Tasks
    if completed_tasks > 0 {
        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_bold, "Completed Tasks", 13.0);
        for t in project.todos.iter().filter(|t| t.completed) {
            let first = format!("[x] {}", t.title);
            for (i, line) in wrap_text(&first, 90).into_iter().enumerate() {
                if i == 0 {
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &line, 12.0);
                } else {
                    let ind = format!("    {}", line);
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 12.0);
                }
            }
            if !t.description.trim().is_empty() {
                let desc = format!("— {}", t.description.trim());
                for line in wrap_text(&desc, 95) {
                    let ind = format!("    {}", line);
                    write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 10.0);
                }
            }
            for s in &t.subtasks {
                let sub = format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title);
                for (i, line) in wrap_text(&sub, 95).into_iter().enumerate() {
                    if i == 0 {
                        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &line, 11.0);
                    } else {
                        let ind = format!("    {}", line);
                        write_line(&doc, &mut current_page, &mut current_layer_idx, &mut cursor_y, margin_left, margin_top, line_height, &font_regular, &ind, 11.0);
                    }
                }
            }
        }
    }

    use std::io::BufWriter;

    // Resolve app-specific external Downloads directory via JNI
    use ndk_context::android_context;
    use jni::objects::{JObject, JValue};
    use jni::JavaVM;

    let ctx = android_context();
    let _out_dir: Option<String> = None;
    unsafe {
        let jvm = JavaVM::from_raw(ctx.vm().cast()).map_err(|e| format!("jvm from raw: {e}"))?;
        let mut env = jvm.attach_current_thread().map_err(|e| format!("attach thread: {e}"))?;
        let activity = JObject::from_raw(ctx.context() as jni::sys::jobject);

        // Prepare to write into public Downloads via MediaStore
        // Build the PDF into memory first
        let mut pdf_buf: Vec<u8> = Vec::new();
        {
            let mut sink = BufWriter::new(&mut pdf_buf);
            doc.save(&mut sink).map_err(|e| format!("save error: {e}"))?;
        }

        // Java: ContentResolver resolver = activity.getContentResolver();
        let resolver: JObject = env
            .call_method(activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[])
            .and_then(|v| v.l())
            .map_err(|e| format!("getContentResolver: {e}"))?;

        // Uri collection = MediaStore.Downloads.EXTERNAL_CONTENT_URI;
        let downloads_class = env.find_class("android/provider/MediaStore$Downloads").map_err(|e| format!("find MediaStore$Downloads: {e}"))?;
        let collection: JObject = env
            .get_static_field(downloads_class, "EXTERNAL_CONTENT_URI", "Landroid/net/Uri;")
            .and_then(|v| v.l())
            .map_err(|e| format!("EXTERNAL_CONTENT_URI: {e}"))?;

        // ContentValues values = new ContentValues();
        let values = env
            .new_object("android/content/ContentValues", "()V", &[])
            .map_err(|e| format!("ContentValues(): {e}"))?;

        // Prepare metadata keys
        let media_cols = env.find_class("android/provider/MediaStore$MediaColumns").map_err(|e| format!("find MediaColumns: {e}"))?;
        let k_display_name: JObject = env.get_static_field(&media_cols, "DISPLAY_NAME", "Ljava/lang/String;").and_then(|v| v.l()).map_err(|e| format!("DISPLAY_NAME: {e}"))?;
        let k_mime_type: JObject = env.get_static_field(&media_cols, "MIME_TYPE", "Ljava/lang/String;").and_then(|v| v.l()).map_err(|e| format!("MIME_TYPE: {e}"))?;
        let k_relative_path: JObject = env.get_static_field(&media_cols, "RELATIVE_PATH", "Ljava/lang/String;").and_then(|v| v.l()).map_err(|e| format!("RELATIVE_PATH: {e}"))?;
        let k_is_pending: JObject = env.get_static_field(&media_cols, "IS_PENDING", "Ljava/lang/String;").and_then(|v| v.l()).map_err(|e| format!("IS_PENDING: {e}"))?;

        let ts = chrono::Utc::now().timestamp();
        let fname = format!("{}_{}.pdf", project.name.replace('/', "-"), ts);
        let v_name = env.new_string(fname).map_err(|e| format!("new_string name: {e}"))?;
        let v_mime = env.new_string("application/pdf").map_err(|e| format!("new_string mime: {e}"))?;
        // Some devices require no trailing slash
        let v_rel = env.new_string("Download").map_err(|e| format!("new_string rel: {e}"))?;
        let one = JValue::Int(1);

        // values.put(key, value)
        env
            .call_method(&values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&k_display_name), JValue::Object(&JObject::from(v_name))])
            .map_err(|e| format!("values.put name: {e}"))?;
        env
            .call_method(&values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&k_mime_type), JValue::Object(&JObject::from(v_mime))])
            .map_err(|e| format!("values.put mime: {e}"))?;
        env
            .call_method(&values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&k_relative_path), JValue::Object(&JObject::from(v_rel))])
            .map_err(|e| format!("values.put rel: {e}"))?;
        // values.put(IS_PENDING, 1)
        let int_one_obj = env
            .call_static_method("java/lang/Integer", "valueOf", "(I)Ljava/lang/Integer;", &[one])
            .and_then(|v| v.l())
            .map_err(|e| format!("Integer.valueOf(1): {e}"))?;
        env
            .call_method(&values, "put", "(Ljava/lang/String;Ljava/lang/Integer;)V", &[JValue::Object(&k_is_pending), JValue::Object(&int_one_obj)])
            .map_err(|e| format!("values.put pending: {e}"))?;

        // Uri uri = resolver.insert(collection, values);
        let uri: JObject = env
            .call_method(&resolver, "insert", "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;", &[JValue::Object(&collection), JValue::Object(&values)])
            .and_then(|v| v.l())
            .map_err(|e| format!("resolver.insert: {e}"))?;
        if uri.is_null() {
            return Err("resolver.insert returned null".into());
        }

        // OutputStream os = resolver.openOutputStream(uri);
        let os: JObject = env
            .call_method(&resolver, "openOutputStream", "(Landroid/net/Uri;)Ljava/io/OutputStream;", &[JValue::Object(&uri)])
            .and_then(|v| v.l())
            .map_err(|e| format!("openOutputStream: {e}"))?;
        if os.is_null() {
            return Err("openOutputStream returned null".into());
        }

        // Write the bytes: os.write(byte[])
        let arr = env.byte_array_from_slice(&pdf_buf).map_err(|e| format!("byte_array_from_slice: {e}"))?;
        let arr_obj = JObject::from(arr);
        env
            .call_method(&os, "write", "([B)V", &[JValue::Object(&arr_obj)])
            .map_err(|e| format!("OutputStream.write: {e}"))?;
        env.call_method(&os, "flush", "()V", &[]).map_err(|e| format!("OutputStream.flush: {e}"))?;
        env.call_method(&os, "close", "()V", &[]).map_err(|e| format!("OutputStream.close: {e}"))?;

        // Mark not pending: values = new ContentValues(); values.put(IS_PENDING, 0); resolver.update(uri, values, null, null);
        let values2 = env.new_object("android/content/ContentValues", "()V", &[]).map_err(|e| format!("ContentValues() #2: {e}"))?;
        let zero = JValue::Int(0);
        let zero_obj = env
            .call_static_method("java/lang/Integer", "valueOf", "(I)Ljava/lang/Integer;", &[zero])
            .and_then(|v| v.l())
            .map_err(|e| format!("Integer.valueOf(0): {e}"))?;
        env
            .call_method(&values2, "put", "(Ljava/lang/String;Ljava/lang/Integer;)V", &[JValue::Object(&k_is_pending), JValue::Object(&zero_obj)])
            .map_err(|e| format!("values2.put pending=0: {e}"))?;
        env
            .call_method(
                &resolver,
                "update",
                "(Landroid/net/Uri;Landroid/content/ContentValues;Ljava/lang/String;[Ljava/lang/String;)I",
                &[
                    JValue::Object(&uri),
                    JValue::Object(&values2),
                    JValue::Object(&JObject::null()),
                    JValue::Object(&JObject::null()),
                ],
            )
            .map_err(|e| format!("resolver.update pending=0: {e}"))?;

        println!("[Export][Android] Saved to MediaStore Downloads");
        return Ok(());
    }
}

// Desktop export (with file dialog when `desktop` feature is enabled)
#[cfg(all(not(target_os = "android"), feature = "desktop"))]
fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    let Some(path) = FileDialog::new()
        .set_title("Export Project to PDF")
        .set_file_name(&format!("{}.pdf", project.name))
        .save_file() else {
        return Err("Save canceled".into());
    };

    // Create doc and fonts
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Project Report — {}", project.name),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("font error: {e}"))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("font error: {e}"))?;

    // Layout
    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let mut cursor_y = Mm(297.0) - margin_top;

    // Helpers
    let mut ensure_page = |doc: &PdfDocumentReference| {
        if cursor_y.0 < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
    };
    let mut write_line = |doc: &PdfDocumentReference, text: &str, size_pt: f64, bold: bool| {
        ensure_page(doc);
        let font = if bold { &font_bold } else { &font_regular };
        current_layer.use_text(text, size_pt, margin_left, cursor_y, font);
        // approximate line height from font size
        cursor_y = Mm(cursor_y.0 - (size_pt * 0.42));
    };
    fn wrap_text(s: &str, max_chars: usize) -> Vec<String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for w in words {
            if line.is_empty() {
                line.push_str(w);
            } else if line.len() + 1 + w.len() <= max_chars {
                line.push(' ');
                line.push_str(w);
            } else {
                lines.push(line);
                line = w.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        if lines.is_empty() { vec![String::new()] } else { lines }
    }

    // Data & Summary
    let total_tasks = project.todos.len();
    let completed_tasks = project.todos.iter().filter(|t| t.completed).count();
    let active_tasks = total_tasks - completed_tasks;
    let total_subtasks: usize = project.todos.iter().map(|t| t.subtasks.len()).sum();
    let completion = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Header
    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    write_line(&doc, &format!("Project Report"), 18.0, true);
    write_line(&doc, &format!("{}", project.name), 15.0, false);
    write_line(&doc, &format!("Generated: {}", date_str), 10.0, false);
    write_line(&doc, "", 8.0, false);

    // Summary block
    write_line(&doc, "Summary", 14.0, true);
    write_line(
        &doc,
        &format!(
            "Tasks: {}  •  Completed: {}  •  Active: {}  •  Subtasks: {}  •  Completion: {:>.1}%",
            total_tasks, completed_tasks, active_tasks, total_subtasks, completion
        ),
        11.0,
        false,
    );
    write_line(&doc, "", 6.0, false);

    // Section: Active Tasks
    if active_tasks > 0 {
        write_line(&doc, "Active Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| !t.completed) {
            for (i, line) in wrap_text(&format!("[ ] {}", t.title), 90).into_iter().enumerate() {
                write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 10.0, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 11.0, false);
                }
            }
        }
        write_line(&doc, "", 6.0, false);
    }

    // Section: Completed Tasks
    if completed_tasks > 0 {
        write_line(&doc, "Completed Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| t.completed) {
            for (i, line) in wrap_text(&format!("[x] {}", t.title), 90).into_iter().enumerate() {
                write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 10.0, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 11.0, false);
                }
            }
        }
    }

    use std::fs::File;
    use std::io::BufWriter;
    let mut out = BufWriter::new(File::create(path).map_err(|e| format!("create error: {e}"))?);
    doc.save(&mut out).map_err(|e| format!("save error: {e}"))?;
    println!("[Export] Saved PDF successfully");
    Ok(())
}

// Desktop export fallback when `desktop` feature is NOT enabled (no file dialog)
#[cfg(all(not(target_os = "android"), not(feature = "desktop")))]
fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    // Create doc and fonts
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Project Report — {}", project.name),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("font error: {e}"))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("font error: {e}"))?;

    // Layout
    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let mut cursor_y = Mm(297.0) - margin_top;

    let mut ensure_page = |doc: &PdfDocumentReference| {
        if cursor_y.0 < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
    };
    let mut write_line = |doc: &PdfDocumentReference, text: &str, size_pt: f64, bold: bool| {
        ensure_page(doc);
        let font = if bold { &font_bold } else { &font_regular };
        current_layer.use_text(text, size_pt, margin_left, cursor_y, font);
        cursor_y = Mm(cursor_y.0 - (size_pt * 0.42));
    };
    fn wrap_text(s: &str, max_chars: usize) -> Vec<String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for w in words {
            if line.is_empty() {
                line.push_str(w);
            } else if line.len() + 1 + w.len() <= max_chars {
                line.push(' ');
                line.push_str(w);
            } else {
                lines.push(line);
                line = w.to_string();
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        if lines.is_empty() { vec![String::new()] } else { lines }
    }

    // Data & Summary
    let total_tasks = project.todos.len();
    let completed_tasks = project.todos.iter().filter(|t| t.completed).count();
    let active_tasks = total_tasks - completed_tasks;
    let total_subtasks: usize = project.todos.iter().map(|t| t.subtasks.len()).sum();
    let completion = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else { 0.0 };

    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    write_line(&doc, &format!("Project Report"), 18.0, true);
    write_line(&doc, &format!("{}", project.name), 15.0, false);
    write_line(&doc, &format!("Generated: {}", date_str), 10.0, false);
    write_line(&doc, "", 8.0, false);

    write_line(&doc, "Summary", 14.0, true);
    write_line(&doc, &format!(
        "Tasks: {}  •  Completed: {}  •  Active: {}  •  Subtasks: {}  •  Completion: {:.1}%",
        total_tasks, completed_tasks, active_tasks, total_subtasks, completion
    ), 11.0, false);
    write_line(&doc, "", 6.0, false);

    if active_tasks > 0 {
        write_line(&doc, "Active Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| !t.completed) {
            for (i, line) in wrap_text(&format!("[ ] {}", t.title), 90).into_iter().enumerate() {
                write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 10.0, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 11.0, false);
                }
            }
        }
        write_line(&doc, "", 6.0, false);
    }

    if completed_tasks > 0 {
        write_line(&doc, "Completed Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| t.completed) {
            for (i, line) in wrap_text(&format!("[x] {}", t.title), 90).into_iter().enumerate() {
                write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 10.0, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line(&doc, if i == 0 { &line } else { &format!("    {}", line) }, 11.0, false);
                }
            }
        }
    }

    use std::fs::File;
    use std::io::BufWriter;
    use std::path::PathBuf;
    // Default path: ~/Downloads/<project>_<ts>.pdf (best-effort)
    let mut dest = if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push("Downloads");
        p
    } else {
        std::env::temp_dir()
    };
    let ts = chrono::Utc::now().timestamp();
    let fname = format!("{}_{}.pdf", project.name.replace('/', "-"), ts);
    dest.push(fname);

    let mut out = BufWriter::new(File::create(&dest).map_err(|e| format!("create error: {e}"))?);
    doc.save(&mut out).map_err(|e| format!("save error: {e}"))?;
    println!("[Export] Saved PDF to {}", dest.display());
    Ok(())
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub projects: Signal<Vec<Project>>,
    pub active_project_id: Signal<Option<u64>>,
    pub new_title: Signal<String>,
    pub editing_id: Signal<Option<u64>>,
    pub editing_text: Signal<String>,
    pub next_id: Signal<u64>,
    pub filter: Signal<Filter>,
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")] Projects {},
    #[route("/list")] List {},
    #[route("/todo/:id")] Details { id: u64 },
}

#[derive(Clone)]
struct FlashState { msg: Signal<Option<String>> }

#[component]
pub fn App() -> Element {
    let mut projects = use_signal(Vec::<Project>::new);
    let mut active_project_id = use_signal(|| Option::<u64>::None);
    let new_title = use_signal(String::new);
    let editing_id = use_signal(|| Option::<u64>::None);
    let editing_text = use_signal(String::new);
    let mut next_id = use_signal(|| 1u64);
    let filter = use_signal(|| Filter::All);

    use_context_provider(|| AppState {
        projects: projects.clone(),
        active_project_id: active_project_id.clone(),
        new_title: new_title.clone(),
        editing_id: editing_id.clone(),
        editing_text: editing_text.clone(),
        next_id: next_id.clone(),
        filter: filter.clone(),
    });
    use_context_provider(|| ProjectsState { projects: projects.clone(), active_project_id: active_project_id.clone() });
    let active_project_snap = use_signal(|| Option::<Project>::None);
    use_context_provider(|| HeaderState { active_project: active_project_snap.clone() });
    // Startup log and toast
    use_effect(move || {
        println!("[App] started");
    });

    use_effect(move || {
        let loaded = load_or_migrate_projects();
        println!("[App] Loaded {} project(s)", loaded.len());
        if !loaded.is_empty() {
            if active_project_id.read().is_none() {
                println!("[App] No active project set. Selecting first: id={} name={}", loaded[0].id, loaded[0].name);
                active_project_id.set(Some(loaded[0].id));
            }
            let max_id_val = loaded.iter().flat_map(|p| p.todos.iter()).fold(0u64, |acc, t| max(acc, t.id));
            next_id.set(max_id_val + 1);
        }
        projects.set(loaded);
    });

    {
        let projects = projects.clone();
        let mut active_project_id = active_project_id.clone();
        let mut active_project_snap = active_project_snap.clone();
        use_effect(move || {
            let opt = active_project_id.read().and_then(|id| projects.read().iter().find(|p| p.id == id).cloned());
            if let Some(ref p) = opt { println!("[HeaderState] Active project snapshot updated: id={} name={}", p.id, p.name); } else { println!("[HeaderState] Active project snapshot updated: None"); }
            active_project_snap.set(opt);
        });
    }

    // global flash banner state
    let flash = use_signal(|| None::<String>);
    // provide to children without a JSX ContextProvider component
    use_context_provider(|| FlashState { msg: flash.clone() });

    // Note: auto-dismiss removed to avoid extra async runtime dependency on Android.

    // Initialize i18n with Russian as default (disabled on Android to avoid startup panic)
    #[cfg(not(target_os = "android"))]
    use_init_i18n(|| {
        I18nConfig::new(langid!("ru-RU"))
            .with_locale(Locale::new_static(langid!("en-US"), include_str!("./i18n/en-US.ftl")))
            .with_locale(Locale::new_static(langid!("ru-RU"), include_str!("./i18n/ru-RU.ftl")))
    });

    rsx! {
        { head_nodes() }
        { if let Some(m) = flash.read().clone() { rsx!{
            div {
                style: "position:fixed; top:12px; left:50%; transform:translateX(-50%); background:rgba(56,189,248,0.95); color:#0b1020; padding:10px 14px; border-radius:8px; box-shadow:0 8px 24px rgba(0,0,0,0.25); font-weight:600; z-index:9999;",
                {m}
            }
        } } else { rsx!{} } }
        { main_body() }
    }
}

#[cfg(target_os = "android")]
fn main_body() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |e| rsx!{ pre { style: "white-space:pre-wrap; padding:12px; color:#ef4444;", "{e:?}" } },
            Router::<Route> {}
        }
    }
}

#[cfg(not(target_os = "android"))]
fn main_body() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |e| rsx!{ pre { style: "white-space:pre-wrap; padding:12px; color:#ef4444;", "{e:?}" } },
            Router::<Route> {}
        }
    }
}

#[component]
pub fn List() -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let active_project_id = state.active_project_id.read();
    let mut new_title = state.new_title;
    let mut editing_id = state.editing_id;
    let mut editing_text = state.editing_text;
    let mut filter = state.filter;
    let mut next_id = state.next_id;
    let nav = use_navigator();
    let mut flash = use_context::<FlashState>();

    #[cfg(target_os = "android")]
    let no_active_lbl = "Активный проект не выбран. Откройте раздел Проекты.";
    #[cfg(not(target_os = "android"))]
    let no_active_lbl = "No active project selected. Go to Projects.";

    let Some(active_id) = *active_project_id else {
        return rsx! { div { class: "app", div { class: "card", "{no_active_lbl}" } } };
    };

    let project = projects.read().iter().find(|p| p.id == active_id).cloned();
    #[cfg(target_os = "android")]
    let not_found_project_lbl = "Проект не найден";
    #[cfg(not(target_os = "android"))]
    let not_found_project_lbl = "Project not found";

    let Some(proj) = project else {
        return rsx! { div { class: "app", div { class: "card", "{not_found_project_lbl}" } } };
    };

    // derive view state and handlers
    let active_filter = *filter.read();
    let count = match active_filter {
        Filter::All => proj.todos.len(),
        Filter::Active => proj.todos.iter().filter(|t| !t.completed).count(),
        Filter::Completed => proj.todos.iter().filter(|t| t.completed).count(),
    };
    let visible: Vec<Todo> = match active_filter {
        Filter::All => proj.todos.clone(),
        Filter::Active => proj.todos.iter().cloned().filter(|t| !t.completed).collect(),
        Filter::Completed => proj.todos.iter().cloned().filter(|t| t.completed).collect(),
    };

    let mut on_switch = move |_| {
        nav.push(Route::Projects {});
    };
    let mut on_export = move |_| {
        println!("[Export] Clicked");
        match export_active_project_pdf(&projects.read(), Some(active_id)) {
            Ok(()) => {
                println!("[Export] Success");
                #[cfg(target_os = "android")]
                flash.msg.set(Some("Экспортировано в Загрузки".to_string()));
                #[cfg(not(target_os = "android"))]
                flash.msg.set(Some("Exported to Downloads".to_string()));
            }
            Err(e) => {
                eprintln!("[Export] Error: {e}");
                flash.msg.set(Some(format!("Export failed: {e}")));
            }
        }
    };
    let mut on_input = move |e: FormEvent| { new_title.set(e.value()); };
    let mut on_enter = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            let title = new_title.read().trim().to_string();
            if !title.is_empty() {
                if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                    let id = *next_id.read();
                    *next_id.write() = id + 1;
                    p.todos.push(Todo { id, title: title.clone(), completed: false, description: String::new(), subtasks: Vec::new() });
                }
                save_projects(&projects.read());
                new_title.set(String::new());
            }
        }
    };
    let mut on_add = move |_| {
        let title = new_title.read().trim().to_string();
        if !title.is_empty() {
            if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                let id = *next_id.read();
                *next_id.write() = id + 1;
                p.todos.push(Todo { id, title: title.clone(), completed: false, description: String::new(), subtasks: Vec::new() });
            }
            save_projects(&projects.read());
            new_title.set(String::new());
        }
    };

    let mut on_all = move |_| { filter.set(Filter::All); };
    let mut on_active_f = move |_| { filter.set(Filter::Active); };
    let mut on_completed_f = move |_| { filter.set(Filter::Completed); };
    let mut on_clear_completed = move |_| {
        if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
            p.todos.retain(|t| !t.completed);
        }
        save_projects(&projects.read());
    };

    rsx! {
        div { class: "app",
            Header { count: count, on_switch: on_switch, on_export: on_export }
            div { class: "card", style: "background:#ffffff; color:#0f172a; border:1px solid rgba(15,23,42,0.12); border-radius:16px; margin:12px auto; padding:18px; box-shadow: 0 10px 25px rgba(0,0,0,0.08);",
                AddForm { value: new_title.read().clone(), on_input: on_input, on_enter: on_enter, on_add: on_add }
                FilterBar { active: active_filter, on_all: on_all, on_active: on_active_f, on_completed: on_completed_f, on_clear_completed: on_clear_completed }
                { if visible.is_empty() {
                    #[cfg(target_os = "android")]
                    let empty_lbl = "Задач пока нет. Добавьте новую выше.";
                    #[cfg(not(target_os = "android"))]
                    let empty_lbl = "No tasks yet. Add one above.";
                    rsx!{ div { style: "opacity:.8; color:var(--muted); padding: 16px 4px;", "{empty_lbl}" } }
                } else {
                    rsx!{ ul { class: "list",
                        for todo in visible.into_iter() {
                            crate::components::todo_item::TodoItem {
                                key: "todo-{todo.id}",
                                todo: todo.clone(),
                                is_editing: matches!(*editing_id.read(), Some(id) if id == todo.id),
                                editing_text: editing_text.read().clone(),
                                on_toggle: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                        if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.completed = !t.completed; }
                                    }
                                    save_projects(&projects.read());
                                },
                                on_start_edit: move |_| { editing_id.set(Some(todo.id)); editing_text.set(todo.title.clone()); },
                                on_remove: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) { p.todos.retain(|t| t.id != todo.id); }
                                    save_projects(&projects.read());
                                },
                                on_save_click: move |_| {
                                    if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                        if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.title = editing_text.read().clone(); }
                                    }
                                    editing_id.set(None);
                                    editing_text.set(String::new());
                                    save_projects(&projects.read());
                                },
                                on_save_key: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        if let Some(p) = projects.write().iter_mut().find(|p| p.id == active_id) {
                                            if let Some(t) = p.todos.iter_mut().find(|t| t.id == todo.id) { t.title = editing_text.read().clone(); }
                                        }
                                        editing_id.set(None);
                                        editing_text.set(String::new());
                                        save_projects(&projects.read());
                                    } else if e.key() == Key::Escape {
                                        editing_id.set(None);
                                        editing_text.set(String::new());
                                    }
                                },
                                on_edit_input: move |e: FormEvent| { editing_text.set(e.value()); },
                                on_cancel: move |_| { editing_id.set(None); editing_text.set(String::new()); },
                                on_drag_start: move |_id: u64| {},
                                on_drag_over: move |_id: u64| {},
                                on_drag_leave: move |_id: u64| {},
                                on_drag_end: move |_id: u64| {},
                                on_drop: move |_id: u64| {},
                                is_dragging: false,
                                is_drag_over: false,
                            }
                        }
                    } }
                } }
            }
        }
    }
}

#[component]
pub fn Projects() -> Element {
    let _ = use_context::<ProjectsState>();
    rsx! { div { class: "app", crate::components::projects::Projects {} } }
}

#[component]
pub fn Details(id: u64) -> Element {
    let state = use_context::<AppState>();
    let mut projects = state.projects;
    let active_project_id = state.active_project_id.read().unwrap_or(0);
    let nav = use_navigator();

    let todo_opt = projects.read().iter().find(|p| p.id == active_project_id).and_then(|p| p.todos.iter().find(|t| t.id == id)).cloned();
    #[cfg(target_os = "android")]
    let not_found_lbl = "Не найдено";
    #[cfg(not(target_os = "android"))]
    let not_found_lbl = "Not found";
    let Some(todo) = todo_opt else { return rsx!{ div { class: "app", div { class: "card", "{not_found_lbl}" } } }; };

    let mut add_sub = move |title: String| {
        if title.trim().is_empty() { return; }
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_project_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            let next_sid = it.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1;
            it.subtasks.push(Subtask { id: next_sid, title, completed: false });
            it.completed = false;
        }
        save_projects(&projects.read());
    };
    let mut toggle_sub = move |sid: u64| {
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_project_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            if let Some(st) = it.subtasks.iter_mut().find(|s| s.id == sid) {
                st.completed = !st.completed;
            }
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            }
        }
        save_projects(&projects.read());
    };
    let mut remove_sub = move |sid: u64| {
        if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_project_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) {
            it.subtasks.retain(|s| s.id != sid);
            if !it.subtasks.is_empty() {
                it.completed = it.subtasks.iter().all(|s| s.completed);
            } else {
                // No subtasks: do not auto-complete; leave as-is
            }
        }
        save_projects(&projects.read());
    };
    let mut update_desc = move |v: String| { if let Some(it) = projects.write().iter_mut().find(|p| p.id == active_project_id).and_then(|p| p.todos.iter_mut().find(|t| t.id == id)) { it.description = v; } save_projects(&projects.read()); };

    let mut sub_input = use_signal(String::new);

    rsx! {
        { head_nodes() }
        div { class: "app",
            div { class: "card",
                div { class: "row between", style: "margin-bottom:12px;",
                    {
                        #[cfg(target_os = "android")]
                        { rsx!{ button { class: "btn btn-ghost", onclick: move |_| { nav.push(Route::List {}); }, "← Назад" } } }
                        #[cfg(not(target_os = "android"))]
                        { rsx!{ button { class: "btn btn-ghost", onclick: move |_| { nav.push(Route::List {}); }, "← Back" } } }
                    }
                }
                h2 { class: "title", "{todo.title}" }
                {
                    #[cfg(target_os = "android")]
                    { rsx!{ textarea { class: "text desc", rows: "3", placeholder: "Описание...", value: "{todo.description}", oninput: move |e| update_desc(e.value()) } } }
                    #[cfg(not(target_os = "android"))]
                    { rsx!{ textarea { class: "text desc", rows: "3", placeholder: "Description...", value: "{todo.description}", oninput: move |e| update_desc(e.value()) } } }
                }
                {
                    #[cfg(target_os = "android")]
                    { rsx!{ h3 { style: "margin-top:16px;", "Подзадачи" } } }
                    #[cfg(not(target_os = "android"))]
                    { rsx!{ h3 { style: "margin-top:16px;", "Subtasks" } } }
                }
                ul { class: "subtasks",
                    for st in todo.subtasks.clone().into_iter() {
                        li { key: "sub-{st.id}", class: "sub-item",
                            input { r#type: "checkbox", checked: st.completed, onclick: move |_| toggle_sub(st.id) }
                            span { class: if st.completed { "sub-title completed" } else { "sub-title" }, "{st.title}" }
                            button { class: "btn btn-ghost sub-remove", onclick: move |_| remove_sub(st.id), "✕" }
                        }
                    }
                    li { class: "sub-add",
                        {
                            #[cfg(target_os = "android")]
                            { rsx!{
                                input { class: "text sub-input", r#type: "text", placeholder: "Добавить подзадачу…", value: "{sub_input.read()}", oninput: move |e| sub_input.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } } } }
                                button { class: "btn btn-primary sub-add-btn", onclick: move |_| { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } }, "Добавить" }
                            } }
                            #[cfg(not(target_os = "android"))]
                            { rsx!{
                                input { class: "text sub-input", r#type: "text", placeholder: "Add a subtask…", value: "{sub_input.read()}", oninput: move |e| sub_input.set(e.value()), onkeydown: move |e| { if e.key() == Key::Enter { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } } } }
                                button { class: "btn btn-primary sub-add-btn", onclick: move |_| { let v = sub_input.read().trim().to_string(); if !v.is_empty() { add_sub(v); sub_input.set(String::new()); } }, "Add" }
                            } }
                        }
                    }
                }
            }
        }
    }
}
