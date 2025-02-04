use super::{GitObject, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileTree<'a> {
    root_dir: PathBuf,
    objects: &'a [GitObject],
}

impl<'a> FileTree<'a> {
    pub fn new<P: AsRef<Path>>(root_dir: P, objects: &'a [GitObject]) -> Self {
        Self {
            root_dir: root_dir.as_ref().into(),
            objects,
        }
    }

    pub fn write_all(self) -> Result<()> {
        for object in self.objects {
            self.write(object)?;
        }
        Ok(())
    }

    fn file_name_of(&self, object: &'a GitObject) -> Option<&'a str> {
        let hash = object.hash();

        self.parent_of(object)
            .and_then(|o| {
                if let GitObject::Tree(ref trees) = o {
                    trees.iter().find(|tree| tree.hash() == hash)
                } else {
                    None
                }
            })
            .map(|tree| tree.name())
    }

    fn parent_of(&self, object: &'a GitObject) -> Option<&'a GitObject> {
        self.objects.iter().find(is_parent_of(object))
    }

    fn ancestors_of(&self, object: &'a GitObject) -> Ancestors<'a> {
        Ancestors::new(object, self.objects)
    }

    fn path_of(&self, object: &'a GitObject) -> PathBuf {
        let mut path = self.root_dir.clone();
        let mut ancestors: Vec<&str> = self
            .ancestors_of(object)
            .filter_map(|o| self.file_name_of(o))
            .collect();
        ancestors.reverse();
        for dirname in ancestors {
            path.push(dirname);
        }
        if let Some(file_name) = self.file_name_of(object) {
            path.push(file_name);
        }
        path
    }

    fn write(&self, object: &'a GitObject) -> Result<()> {
        if let Some(parent) = self.parent_of(object) {
            self.write(parent)?;
        }

        let path = self.path_of(object);

        if !path.exists() {
            if let GitObject::Blob(ref blob) = object {
                let mut f = File::create(path)?;
                f.write_all(blob.as_ref())?;
            } else if let GitObject::Tree(_) = object {
                fs::create_dir(path)?;
            }
        }

        Ok(())
    }
}

fn is_parent_of(object: &GitObject) -> Box<dyn FnMut(&&GitObject) -> bool> {
    let hash = object.hash();

    Box::new(move |o| {
        if let GitObject::Tree(trees) = o {
            trees.iter().any(|tree| tree.hash() == hash)
        } else {
            false
        }
    })
}

#[derive(Debug)]
struct Ancestors<'a> {
    current: &'a GitObject,
    source: &'a [GitObject],
}

impl<'a> Ancestors<'a> {
    fn new(current: &'a GitObject, source: &'a [GitObject]) -> Self {
        Self { current, source }
    }
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = &'a GitObject;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.source.iter().find(is_parent_of(self.current));

        if let Some(object) = item {
            self.current = object;
        }

        item
    }
}
