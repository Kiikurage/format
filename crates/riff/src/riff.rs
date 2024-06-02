use std::fs::File;
use std::io::Error;
use std::os::unix::fs::FileExt;
use std::path::Path;

#[derive(Debug)]
pub struct Chunk {
    pub id: String,
    pub size: usize,
    pub form_type: String,
    pub children: Vec<Chunk>,
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn list(form_type: impl Into<String>, children: Vec<Chunk>) -> Chunk {
        Self::list_with_id("LIST", form_type, children)
    }

    pub fn list_with_id(
        id: impl Into<String>,
        form_type: impl Into<String>,
        children: Vec<Chunk>,
    ) -> Chunk {
        Chunk {
            id: id.into(),
            size: 4 + children.iter().map(|c| c.size + 8).sum::<usize>(),
            form_type: form_type.into(),
            children,
            data: Vec::new(),
        }
    }

    pub fn new(id: impl Into<String>, data: Vec<u8>) -> Chunk {
        Chunk {
            id: id.into(),
            size: data.len(),
            form_type: String::new(),
            children: Vec::new(),
            data,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Chunk, Error> {
        let mut file = File::open(path)?;
        Ok(Chunk::read(&mut file))
    }

    pub fn read(file: &mut File) -> Chunk {
        parse_chunk(file, 0).unwrap()
    }

    pub fn print(&self) {
        self.print_with_indent("".to_string());
    }

    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.id.as_bytes());
        out.extend_from_slice((self.size as u32).to_le_bytes().as_ref());

        if self.children.is_empty() {
            out.extend_from_slice(&self.data);
        } else {
            out.extend_from_slice(self.form_type.as_bytes());
            for child in self.children.iter() {
                child.write(out);
            }
        }
    }

    fn print_with_indent(&self, indent: String) {
        let line_length = 50;

        let id = if self.form_type.is_empty() {
            self.id.clone()
        } else {
            format!("{}:{}({})", self.id, self.form_type, self.children.len())
        };

        let size = self.size.to_string();

        println!(
            "{}{} {}{}",
            color(&indent, 239),
            id,
            color(
                &".".repeat(
                    line_length
                        - indent.chars().count()
                        - id.chars().count()
                        - size.chars().count()
                ),
                239
            ),
            color(&size, 6)
        );

        let indent = indent.replace(" ├──", " │  ").replace(" └──", "    ");
        for i in 0..self.children.len() {
            let child = &self.children[i];
            let new_indent = if i == self.children.len() - 1 {
                indent.clone() + " └──"
            } else {
                indent.clone() + " ├──"
            };
            child.print_with_indent(new_indent);
        }
    }
}

fn parse_chunk(file: &mut File, mut offset: usize) -> Result<Chunk, std::io::Error> {
    let mut buf4 = [0u8; 4];

    file.read_at(&mut buf4, offset as u64)?;
    let id = String::from_utf8_lossy(&buf4).to_string();
    offset += 4;

    file.read_at(&mut buf4, offset as u64)?;
    let size = u32::from_le_bytes(buf4) as usize;
    offset += 4;

    let mut form_type = String::new();

    let (children, data) = match id.as_str() {
        "RIFF" | "LIST" => {
            file.read_at(&mut buf4, offset as u64)?;
            form_type = String::from_utf8_lossy(&buf4).to_string();
            offset += 4;
            (parse_chunk_list(file, offset, size - 4)?, Vec::new())
        }
        _ => {
            let mut buf = vec![0u8; size as usize];
            file.read_at(&mut buf, offset as u64)?;
            (Vec::new(), buf)
        }
    };

    Ok(Chunk {
        id,
        size,
        form_type,
        children,
        data,
    })
}

fn parse_chunk_list(
    file: &mut File,
    offset: usize,
    size: usize,
) -> Result<Vec<Chunk>, std::io::Error> {
    let mut chunks = Vec::new();
    let mut offset = offset;
    let end = offset + size;

    while offset < end {
        let chunk = parse_chunk(file, offset)?;
        // Odd size chunks are padded with a null byte
        offset += chunk.size + (chunk.size & 1) + 8; // 8 for id and size fields
        chunks.push(chunk);
    }

    Ok(chunks)
}

fn color(text: &String, color: u8) -> String {
    format!("\x1b[38;5;{}m{}\x1b[m", color, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chunk() -> Result<(), Error> {
        let riff = Chunk::open("../../resources/sample.avi")?;

        riff.print();

        Ok(())
    }
}
