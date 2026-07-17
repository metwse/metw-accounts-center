use std::{collections::HashMap, env, fs::File, io::Write, path::Path};

pub fn minimize_email_templates() {
    println!("cargo:rerun-if-changed=email_templates/emails.toml");
    println!("cargo:rerun-if-changed=email_templates/template.html");
    println!("cargo:rerun-if-changed=email_templates/template.txt");

    let template_arguments = include_str!("../email_templates/emails.toml");

    let html_template: &str = include_str!("../email_templates/template.html");
    let text_template: &str = include_str!("../email_templates/template.txt");

    let html_template = String::from_utf8(minify_html::minify(
        html_template.as_bytes(),
        &minify_html::Cfg::default(),
    ))
    .unwrap();

    let raw_templates =
        toml::from_str::<HashMap<String, HashMap<String, String>>>(template_arguments).unwrap();

    let mut templates = HashMap::<String, HashMap<String, String>>::new();

    for (template_name, value) in raw_templates.into_iter() {
        templates.insert(template_name, value);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    for (template_name, mut value) in templates.into_iter() {
        value.insert(
            "callback_not_minimized".to_string(),
            r#"href="{callback_url}{token}""#.to_string(),
        );

        let html = strfmt::strfmt(&html_template, &value).unwrap();
        let text = strfmt::strfmt(text_template, &value).unwrap();
        let subject = value.remove("subject").unwrap();

        let html_file = out_dir.join(format!("email-templates_{template_name}.html"));
        let text_file = out_dir.join(format!("email-templates_{template_name}.txt"));
        let subject_file = out_dir.join(format!("email-templates_{template_name}-subject.txt"));

        write!(File::create(&html_file).unwrap(), "{html}").unwrap();
        write!(File::create(&text_file).unwrap(), "{text}").unwrap();
        write!(File::create(&subject_file).unwrap(), "{subject}").unwrap();
    }
}
