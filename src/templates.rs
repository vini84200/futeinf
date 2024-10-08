use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*.html";
        let mut tera = Tera::new(source).expect("failed to compile template");
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}
