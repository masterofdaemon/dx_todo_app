#[cfg(target_os = "android")]
mod export_android;

use crate::models::Project;

// Android is implemented in export_android.rs to avoid terminal color artifacts in patches
#[cfg(target_os = "android")]
pub use export_android::export_active_project_pdf;

#[cfg(target_os = "android")]
pub fn _android_pdf_export_placeholder(_: &Vec<Project>, _: Option<u64>) -> Result<(), String> {
    Ok(())
}

// Desktop and other non-Android targets: real PDF export using printpdf built-in fonts
#[cfg(all(not(target_os = "android"), feature = "desktop"))]
pub fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    export_pdf_impl(projects, active_id)
}

#[cfg(all(not(target_os = "android"), not(feature = "desktop")))]
pub fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    export_pdf_impl(projects, active_id)
}

#[cfg(not(target_os = "android"))]
fn export_pdf_impl(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    use chrono::Local;
    use printpdf::*;
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::PathBuf;

    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    // Create a new A4 document
    let (doc, page1, layer1) = PdfDocument::new(
        format!("Project Report — {}", project.name),
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

    let mut current_page = page1;
    let mut current_layer = doc.get_page(current_page).get_layer(layer1);
    let margin_left = Mm(15.0);
    let margin_top = Mm(15.0);
    let mut cursor_y = Mm(297.0) - margin_top;
    let mut write_line = |doc: &PdfDocumentReference, text: &str, size_pt: f64, bold: bool| {
        if cursor_y.0 < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
        let font = if bold { &font_bold } else { &font_regular };
        current_layer.use_text(text, size_pt, margin_left, cursor_y, font);
        cursor_y = Mm(cursor_y.0 - (size_pt * 0.45));
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

    let total_tasks = project.todos.len();
    let completed_tasks = project.todos.iter().filter(|t| t.completed).count();
    let active_tasks = total_tasks - completed_tasks;
    let total_subtasks: usize = project.todos.iter().map(|t| t.subtasks.len()).sum();
    let completion = if total_tasks > 0 { (completed_tasks as f64 / total_tasks as f64) * 100.0 } else { 0.0 };

    let date_str = Local::now().format("%Y-%m-%d %H:%M").to_string();
    write_line(&doc, &format!("Project Report"), 20.7, true);
    write_line(&doc, &format!("{}", project.name), 17.3, false);
    write_line(&doc, &format!("Generated: {}", date_str), 11.5, false);
    write_line(&doc, "", 9.2, false);

    write_line(&doc, "Summary", 16.1, true);
    write_line(
        &doc,
        &format!(
            "Tasks: {}  •  Completed: {}  •  Active: {}  •  Subtasks: {}  •  Completion: {:.1}%",
            total_tasks, completed_tasks, active_tasks, total_subtasks, completion
        ),
        12.7,
        false,
    );
    write_line(&doc, "", 6.9, false);

    if active_tasks > 0 {
        write_line(&doc, "Active Tasks", 15.0, true);
        for t in project.todos.iter().filter(|t| !t.completed) {
            for (i, line) in wrap_text(&format!("[ ] {}", t.title), 90).into_iter().enumerate() {
                if i == 0 {
                    write_line(&doc, &line, 13.8, false);
                } else {
                    write_line(&doc, &format!("    {}", line), 13.8, false);
                }
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 11.5, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    if i == 0 {
                        write_line(&doc, &line, 12.7, false);
                    } else {
                        write_line(&doc, &format!("    {}", line), 12.7, false);
                    }
                }
            }
        }
        write_line(&doc, "", 6.9, false);
    }

    if completed_tasks > 0 {
        write_line(&doc, "Completed Tasks", 15.0, true);
        for t in project.todos.iter().filter(|t| t.completed) {
            for (i, line) in wrap_text(&format!("[x] {}", t.title), 90).into_iter().enumerate() {
                if i == 0 {
                    write_line(&doc, &line, 13.8, false);
                } else {
                    write_line(&doc, &format!("    {}", line), 13.8, false);
                }
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 11.5, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    if i == 0 {
                        write_line(&doc, &line, 12.7, false);
                    } else {
                        write_line(&doc, &format!("    {}", line), 12.7, false);
                    }
                }
            }
        }
    }

    // Save to ~/Downloads/<project>_<ts>.pdf or tmp if HOME not set
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

    let out = File::create(&dest).map_err(|e| format!("create error: {e}"))?;
    let mut buf = BufWriter::new(out);
    doc.save(&mut buf).map_err(|e| format!("save error: {e}"))?;
    println!("[Export] Saved PDF to {}", dest.display());
    Ok(())
}
