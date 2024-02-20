use std::path::Path;

use oxc::allocator::Allocator;
use oxc::ast::{AstKind, Visit};
use oxc::parser::Parser;
use oxc::span::SourceType;

#[derive(Debug, Default, serde::Serialize)]
struct ImportSpecifier {
    source_name: String,
    local_name: String,
}

#[derive(Debug, Default, serde::Serialize)]
struct ImportItem {
    source: String,
    specifiers: Vec<ImportSpecifier>,
}

#[derive(Debug, Default, serde::Serialize)]
struct ModuleFile {
    path: String,
    imports: Vec<ImportItem>,
    exports: Vec<String>,
    default_export: Option<String>,
}

impl<'a> Visit<'a> for ModuleFile {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::ImportDeclaration(import) => {
                let from = &import.source.value;
                self.imports.push(ImportItem {
                    source: from.to_string(),
                    specifiers: Vec::new(),
                });
            }
            AstKind::ImportSpecifier(import) => {
                let local_name = &import.local.name;
                let imported_name = &import.imported.name();
                let last = self.imports.last_mut().unwrap();
                last.specifiers.push(ImportSpecifier {
                    source_name: imported_name.to_string(),
                    local_name: local_name.to_string(),
                });
            }
            AstKind::ImportDefaultSpecifier(import) => {
                let local = &import.local.name;
                let last = self.imports.last_mut().unwrap();
                last.specifiers.push(ImportSpecifier {
                    source_name: "default".to_string(),
                    local_name: local.to_string(),
                });
            }
            AstKind::ImportNamespaceSpecifier(import) => {
                let local = &import.local.name;
                let last = self.imports.last_mut().unwrap();
                last.specifiers.push(ImportSpecifier {
                    source_name: "*".to_string(),
                    local_name: local.to_string(),
                });
            }
            _ => {}
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
struct Project {
    files: Vec<ModuleFile>,
}

impl Project {
    fn traverse(&mut self, files: Vec<String>) {
        for file in files {
            if self.files.iter().any(|f| f.path == file) {
                continue;
            }
            let path = Path::new(&file);
            // if file is not found, skip it
            if !path.exists() {
                println!("file not found: {}", file);
                continue;
            }
            let source_text = std::fs::read_to_string(path).unwrap();
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(path).unwrap();
            let ret = Parser::new(&allocator, &source_text, source_type).parse();

            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }

            let program = ret.program;

            let mut ast_pass = ModuleFile {
                path: path.to_str().unwrap().to_string(),
                ..Default::default()
            };
            ast_pass.visit_program(&program);
            let imports = ast_pass
                .imports
                .iter()
                .map(|i| i.source.clone())
                .collect::<Vec<_>>();
            self.traverse(imports.clone());
            self.files.push(ast_pass);
        }
    }
}

fn main() {
    let mut project = Project::default();
    let files = ["test.ts"].iter().map(|s| s.to_string()).collect();

    project.traverse(files);

    println!("{}", serde_json::to_string_pretty(&project).unwrap());
}
