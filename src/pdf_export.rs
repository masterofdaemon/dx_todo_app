use crate::models::Project;

// Minimal, compile-safe stubs after refactor. Full PDF export can be re-added later.
#[allow(dead_code)]
pub fn android_toast(_msg: &str) {}

#[cfg(target_os = "android")]
pub fn export_active_project_pdf(_projects: &Vec<Project>, _active_id: Option<u64>) -> Result<(), String> {
    Err("PDF export temporarily disabled after refactor (Android)".into())
}

#[cfg(all(not(target_os = "android"), feature = "desktop"))]
pub fn export_active_project_pdf(_projects: &Vec<Project>, _active_id: Option<u64>) -> Result<(), String> {
    Err("PDF export temporarily disabled after refactor (desktop)".into())
}

#[cfg(all(not(target_os = "android"), not(feature = "desktop")))]
pub fn export_active_project_pdf(_projects: &Vec<Project>, _active_id: Option<u64>) -> Result<(), String> {
    Err("PDF export temporarily disabled after refactor".into())
}
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    let (doc, page1, layer1) = PdfDocument::new(
        [38;5;0m&[0mformat!("Project Report — {}", project.name),
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

    let mut ensure_page = |doc: [38;5;0m&[0mPdfDocumentReference| {
        if cursor_y.0 [38;5;0m<[0m 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            current_page = p;
            current_layer = doc.get_page(current_page).get_layer(l);
            cursor_y = Mm(297.0) - margin_top;
        }
    };
    let mut write_line = |doc: [38;5;0m&[0mPdfDocumentReference, text: [38;5;0m&[0mstr, size_pt: f64, bold: bool| {
        ensure_page(doc);
        let font = if bold { [38;5;0m&[0mfont_bold } else { [38;5;0m&[0mfont_regular };
        current_layer.use_text(text, size_pt, margin_left, cursor_y, font);
        cursor_y = Mm(cursor_y.0 - (size_pt * 0.42));
    };
    fn wrap_text(s: [38;5;0m&[0mstr, max_chars: usize) -[38;5;0m>[0m Vec[38;5;0m<[0mString[38;5;0m>[0m {
        let words: Vec[38;5;0m<[0m[38;5;0m&[0mstr[38;5;0m>[0m = s.split_whitespace().collect();
        let mut lines: Vec[38;5;0m<[0mString[38;5;0m>[0m = Vec::new();
        let mut line = String::new();
        for w in words {
            if line.is_empty() {
                line.push_str(w);
            } else if line.len() + 1 + w.len() [38;5;0m<=[0m max_chars {
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
    let completion = if total_tasks [38;5;0m>[0m 0 { (completed_tasks as f64 / total_tasks as f64) * 100.0 } else { 0.0 };

    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    write_line([38;5;0m&[0mdoc, [38;5;0m&[0mformat!("Project Report"), 18.0, true);
    write_line([38;5;0m&[0mdoc, [38;5;0m&[0mformat!("{}", project.name), 15.0, false);
    write_line([38;5;0m&[0mdoc, [38;5;0m&[0mformat!("Generated: {}", date_str), 10.0, false);
    write_line([38;5;0m&[0mdoc, "", 8.0, false);

    write_line([38;5;0m&[0mdoc, "Summary", 14.0, true);
    write_line(
        [38;5;0m&[0mdoc,
        [38;5;0m&[0mformat!(
            "Tasks: {}  •  Completed: {}  •  Active: {}  •  Subtasks: {}  •  Completion: {:.1}%",
            total_tasks, completed_tasks, active_tasks, total_subtasks, completion
        ),
        11.0,
        false,
    );
    write_line([38;5;0m&[0mdoc, "", 6.0, false);

    if active_tasks [38;5;0m>[0m 0 {
        write_line([38;5;0m&[0mdoc, "Active Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| !t.completed) {
            for (i, line) in wrap_text([38;5;0m&[0mformat!("[ ] {}", t.title), 90).into_iter().enumerate() {
                write_line([38;5;0m&[0mdoc, if i == 0 { [38;5;0m&[0mline } else { [38;5;0m&[0mformat!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text([38;5;0m&[0mformat!("— {}", t.description.trim()), 95) {
                    write_line([38;5;0m&[0mdoc, [38;5;0m&[0mformat!("    {}", line), 10.0, false);
                }
            }
            for s in [38;5;0m&[0mt.subtasks {
                for (i, line) in wrap_text([38;5;0m&[0mformat!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line([38;5;0m&[0mdoc, if i == 0 { [38;5;0m&[0mline } else { [38;5;0m&[0mformat!("    {}", line) }, 11.0, false);
                }
            }
        }
        write_line([38;5;0m&[0mdoc, "", 6.0, false);
    }

    if completed_tasks [38;5;0m>[0m 0 {
        write_line([38;5;0m&[0mdoc, "Completed Tasks", 13.0, true);
        for t in project.todos.iter().filter(|t| t.completed) {
            for (i, line) in wrap_text([38;5;0m&[0mformat!("[x] {}", t.title), 90).into_iter().enumerate() {
                write_line([38;5;0m&[0mdoc, if i == 0 { [38;5;0m&[0mline } else { [38;5;0m&[0mformat!("    {}", line) }, 12.0, false);
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text([38;5;0m&[0mformat!("— {}", t.description.trim()), 95) {
                    write_line([38;5;0m&[0mdoc, [38;5;0m&[0mformat!("    {}", line), 10.0, false);
                }
            }
            for s in [38;5;0m&[0mt.subtasks {
                for (i, line) in wrap_text([38;5;0m&[0mformat!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    write_line([38;5;0m&[0mdoc, if i == 0 { [38;5;0m&[0mline } else { [38;5;0m&[0mformat!("    {}", line) }, 11.0, false);
                }
            }
        }
    }

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

    let mut out = BufWriter::new(File::create([38;5;0m&[0mdest).map_err(|e| format!("create error: {e}"))?);
    doc.save([38;5;0m&[0mmut out).map_err(|e| format!("save error: {e}"))?;
    println!("[Export] Saved PDF to {}", dest.display());
    Ok(())
}
