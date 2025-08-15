use crate::models::Project;

// Android-only: export PDF to app's internal files dir (Context.getFilesDir()).
// This avoids storage permissions. The file can later be shared via an intent if desired.
#[cfg(target_os = "android")]
pub fn export_active_project_pdf(projects: &Vec<Project>, active_id: Option<u64>) -> Result<(), String> {
    use jni::{objects::{JObject, JString, JValue}, JavaVM};
    use ndk_context::android_context;
    use printpdf::*;
    use std::fs::{self, File};
    use std::io::BufWriter;
    use std::path::PathBuf;

    let active_id = active_id.ok_or_else(|| "No active project selected".to_string())?;
    let project = projects
        .iter()
        .find(|p| p.id == active_id)
        .ok_or_else(|| "Active project not found".to_string())?;

    // Build the PDF
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

    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
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
                if i == 0 { write_line(&doc, &line, 13.8, false); }
                else { write_line(&doc, &format!("    {}", line), 13.8, false); }
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 11.5, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    if i == 0 { write_line(&doc, &line, 12.7, false); }
                    else { write_line(&doc, &format!("    {}", line), 12.7, false); }
                }
            }
        }
        write_line(&doc, "", 6.9, false);
    }

    if completed_tasks > 0 {
        write_line(&doc, "Completed Tasks", 15.0, true);
        for t in project.todos.iter().filter(|t| t.completed) {
            for (i, line) in wrap_text(&format!("[x] {}", t.title), 90).into_iter().enumerate() {
                if i == 0 { write_line(&doc, &line, 13.8, false); }
                else { write_line(&doc, &format!("    {}", line), 13.8, false); }
            }
            if !t.description.trim().is_empty() {
                for line in wrap_text(&format!("— {}", t.description.trim()), 95) {
                    write_line(&doc, &format!("    {}", line), 11.5, false);
                }
            }
            for s in &t.subtasks {
                for (i, line) in wrap_text(&format!("    {} {}", if s.completed { "[x]" } else { "[ ]" }, s.title), 95).into_iter().enumerate() {
                    if i == 0 { write_line(&doc, &line, 12.7, false); }
                    else { write_line(&doc, &format!("    {}", line), 12.7, false); }
                }
            }
        }
    }

    // Save via MediaStore into user's Documents folder and make it visible in Files
    // We first serialize the PDF into memory, then write it through ContentResolver.
    let mut pdf_bytes: Vec<u8> = Vec::new();
    {
        let mut buf = BufWriter::new(&mut pdf_bytes);
        doc.save(&mut buf).map_err(|e| format!("save error: {e}"))?;
    }

    unsafe {
        use jni::objects::{JClass};
        let ctx = android_context();
        let jvm = JavaVM::from_raw(ctx.vm().cast()).map_err(|e| format!("jvm from raw: {e}"))?;
        let mut env = jvm.attach_current_thread().map_err(|e| format!("attach thread: {e}"))?;
        let activity = JObject::from_raw(ctx.context() as jni::sys::jobject);

        // Build ContentValues
        let values = env
            .new_object("android/content/ContentValues", "()V", &[])
            .map_err(|e| format!("ContentValues new: {e}"))?;
        let k_name = env.new_string("_display_name").map_err(|e| format!("new_string: {e}"))?;
        let k_type = env.new_string("mime_type").map_err(|e| format!("new_string: {e}"))?;
        let k_rel = env.new_string("relative_path").map_err(|e| format!("new_string: {e}"))?;
        let ts = chrono::Utc::now().timestamp();
        let fname = env
            .new_string(format!("{}_{}.pdf", project.name.replace('/', "-"), ts))
            .map_err(|e| format!("new_string fname: {e}"))?;
        let mime = env.new_string("application/pdf").map_err(|e| format!("new_string mime: {e}"))?;
        let relp = env
            .new_string("Documents/")
            .map_err(|e| format!("new_string rel: {e}"))?;
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::from(&k_name),
                JValue::from(&fname),
            ],
        )
        .map_err(|e| format!("values.put name: {e}"))?;
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::from(&k_type),
                JValue::from(&mime),
            ],
        )
        .map_err(|e| format!("values.put type: {e}"))?;
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::from(&k_rel),
                JValue::from(&relp),
            ],
        )
        .map_err(|e| format!("values.put rel: {e}"))?;

        // resolver = activity.getContentResolver()
        let resolver = env
            .call_method(&activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[])
            .and_then(|v| v.l())
            .map_err(|e| format!("getContentResolver: {e}"))?;

        // Build collection URI for Documents via MediaStore.Files on the primary external volume
        // volume = MediaStore.VOLUME_EXTERNAL_PRIMARY
        let ms_cls: JClass = env
            .find_class("android/provider/MediaStore")
            .map_err(|e| format!("find MediaStore: {e}"))?;
        let vol: JObject = env
            .get_static_field(ms_cls, "VOLUME_EXTERNAL_PRIMARY", "Ljava/lang/String;")
            .and_then(|v| v.l())
            .map_err(|e| format!("VOLUME_EXTERNAL_PRIMARY: {e}"))?;
        // uriColl = MediaStore.Files.getContentUri(volume)
        let files_cls: JClass = env
            .find_class("android/provider/MediaStore$Files")
            .map_err(|e| format!("find MediaStore$Files: {e}"))?;
        let uri_coll = env
            .call_static_method(
                files_cls,
                "getContentUri",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::from(&vol)],
            )
            .and_then(|v| v.l())
            .map_err(|e| format!("Files.getContentUri: {e}"))?;

        // uri = resolver.insert(uriColl, values)
        let uri = env
            .call_method(
                &resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[
                    JValue::from(&uri_coll),
                    JValue::from(&values),
                ],
            )
            .and_then(|v| v.l())
            .map_err(|e| format!("resolver.insert: {e}"))?;

        // out = resolver.openOutputStream(uri)
        let out_stream = env
            .call_method(
                &resolver,
                "openOutputStream",
                "(Landroid/net/Uri;)Ljava/io/OutputStream;",
                &[
                    JValue::from(&uri),
                ],
            )
            .and_then(|v| v.l())
            .map_err(|e| format!("openOutputStream: {e}"))?;

        // Write bytes
        let jbytes = env.byte_array_from_slice(&pdf_bytes).map_err(|e| format!("byte_array_from_slice: {e}"))?;
        let jbytes_obj = JObject::from(jbytes);
        env.call_method(&out_stream, "write", "([B)V", &[
            JValue::from(&jbytes_obj),
        ])
            .map_err(|e| format!("OutputStream.write: {e}"))?;
        env.call_method(&out_stream, "flush", "()V", &[])
            .map_err(|e| format!("OutputStream.flush: {e}"))?;
        env.call_method(&out_stream, "close", "()V", &[])
            .map_err(|e| format!("OutputStream.close: {e}"))?;

        // Optional: prompt a viewer so user sees it in a file app
        let intent = env
            .new_object("android/content/Intent", "()V", &[])
            .map_err(|e| format!("Intent new: {e}"))?;
        let action = env.new_string("android.intent.action.VIEW").map_err(|e| format!("new_string action: {e}"))?;
        env.call_method(&intent, "setAction", "(Ljava/lang/String;)Landroid/content/Intent;", &[
            JValue::from(&action),
        ])
            .map_err(|e| format!("Intent.setAction: {e}"))?;
        let mime = env.new_string("application/pdf").map_err(|e| format!("new_string mime2: {e}"))?;
        env.call_method(
            &intent,
            "setDataAndType",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/content/Intent;",
            &[
                JValue::from(&uri),
                JValue::from(&mime),
            ],
        )
        .map_err(|e| format!("Intent.setDataAndType: {e}"))?;
        // Grant read permission
        env.call_method(&intent, "addFlags", "(I)Landroid/content/Intent;", &[
            JValue::from(1 /* FLAG_GRANT_READ_URI_PERMISSION */),
        ])
            .map_err(|e| format!("Intent.addFlags: {e}"))?;
        // Best-effort: start activity; if none can handle, ignore error
        let _ = env.call_method(&activity, "startActivity", "(Landroid/content/Intent;)V", &[
            JValue::from(&intent),
        ]);
    }

    Ok(())
}

