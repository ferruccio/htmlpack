use html5ever::driver::ParseOpts;
use html5ever::interface::Attribute;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::{Tendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{namespace_url, ns, parse_document, serialize, LocalNameStaticSet};
use quick_error::quick_error;
use std::default::Default;
use std::io::{Read, Write};
use std::path::PathBuf;
use string_cache::Atom;

quick_error! {
    #[derive(Debug)]
    pub enum PackError {
        IoError(err: std::io::Error) {
            from()
            display("I/O Error: {}", err)
        }
    }
}

pub type PackResult<T> = std::result::Result<T, PackError>;

#[derive(Debug)]
pub struct Packer {
    input_path: PathBuf,
    outdir: PathBuf,
    search_paths: Vec<PathBuf>,
    overwrite: bool,
}

impl Packer {
    pub fn new(outdir: PathBuf, search_paths: Vec<PathBuf>, overwrite: bool) -> Packer {
        Packer {
            input_path: PathBuf::new(),
            outdir,
            search_paths,
            overwrite,
        }
    }

    pub fn pack(&mut self, input: PathBuf) -> PackResult<()> {
        self.input_path = match input.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::new(),
        };
        let mut src_file = std::fs::File::open(&input)?;
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut src_file)?;

        self.walk(dom.document.clone());

        let mut destination = self.outdir.clone();
        destination.push(input.iter().last().unwrap());

        if !self.overwrite && destination.exists() {
            println!(
                "output file already exists: {}, use -w to overwrite it",
                destination.to_string_lossy()
            );
        } else {
            let mut dst_file = std::fs::File::create(destination)?;
            dst_file.write_all(b"<!DOCTYPE html>\n")?;
            serialize(&mut dst_file, &dom.document, Default::default())?;
        }
        Ok(())
    }

    fn walk(&self, node: Handle) {
        match node.data {
            NodeData::Document => {}
            NodeData::Doctype { .. } => {}
            NodeData::Text { .. } => {}
            NodeData::Comment { .. } => {}
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                assert!(name.ns == ns!(html));
                let img = Atom::<LocalNameStaticSet>::from("img");
                if name.local == img {
                    let src = Atom::<LocalNameStaticSet>::from("src");
                    for mut attr in attrs.borrow_mut().iter_mut() {
                        if attr.name.local == src {
                            self.update_img_attr(&mut attr);
                        }
                    }
                }
            }
            NodeData::ProcessingInstruction { .. } => {}
        }

        for child in node.children.borrow().iter() {
            self.walk(child.clone());
        }
    }

    fn update_img_attr(&self, attr: &mut Attribute) {
        let value = &attr.value.to_string();
        if let Some(image) = self.find_image(PathBuf::from(&value)) {
            if let Some(data) = self.replace_image(image) {
                attr.value = Tendril::from_slice(&data[..]);
            }
        } else {
            println!("not found: img src={}", value);
        }
    }

    fn find_image(&self, src: PathBuf) -> Option<PathBuf> {
        let mut filename = self.input_path.clone();
        filename.push(&src);
        if filename.exists() {
            return Some(filename);
        }
        for path in &self.search_paths {
            let mut newpath = path.clone();
            newpath.push(&src);
            if newpath.exists() {
                return Some(newpath);
            }
        }
        None
    }

    fn replace_image(&self, src: PathBuf) -> Option<String> {
        let mime = mime_guess::guess_mime_type(&src);
        if let Ok(mut file) = std::fs::File::open(src) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                return Some(format!("data:{};base64,{}", mime, base64::encode(&buffer)));
            }
        }
        None
    }
}
