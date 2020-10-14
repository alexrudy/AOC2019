use anyhow::{anyhow, Error};
use std::fmt;
use std::io::Read;

#[derive(Debug)]
struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Image {
    fn read(shape: (usize, usize), input: Box<dyn Read + 'static>) -> Result<Image, Error> {
        let data = input
            .bytes()
            .map(|item| item.map(|char| char - '0' as u8))
            .collect::<Result<Vec<u8>, std::io::Error>>()?;
        Ok(Image {
            width: shape.0,
            height: shape.1,
            data: data,
        })
    }

    fn layer(&self, n: usize) -> &[u8] {
        let start = (self.width * self.height) * n;
        let end = (self.width * self.height) * (n + 1);
        &self.data[start..end]
    }

    fn nlayers(&self) -> usize {
        self.data.len() / (self.width * self.height)
    }

    fn layers(&self) -> ImageLayerIterator {
        ImageLayerIterator {
            image: self,
            layer: 0,
        }
    }

    fn render(&self) -> RenderedImage {
        let n = self.nlayers();
        let stride = self.width * self.height;
        let mut image = Vec::with_capacity(stride);

        for i in 0..stride {
            let mut pixel = 2;
            for l in 0..n {
                let idx = i + (l * stride);
                pixel = self.data[idx];
                if pixel != 2 {
                    break;
                }
            }
            image.push(pixel);
        }

        RenderedImage {
            width: self.width,
            height: self.height,
            data: image,
        }
    }
}

#[derive(Debug)]
struct RenderedImage {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl fmt::Display for RenderedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let px = self.data[col + (row * self.width)];
                write!(
                    f,
                    "{}",
                    match px {
                        0 => " ",
                        1 => "#",
                        _ => "?",
                    }
                )?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ImageLayerIterator<'i> {
    image: &'i Image,
    layer: usize,
}

impl<'i> Iterator for ImageLayerIterator<'i> {
    type Item = &'i [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.layer == self.image.nlayers() - 1 {
            return None;
        }

        let layer = self.layer;
        self.layer += 1;

        Some(self.image.layer(layer))
    }
}

pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let image = Image::read((25, 6), input)?;

    let layer = image
        .layers()
        .min_by_key(|&layer| layer.iter().filter(|&element| element == &0).count())
        .ok_or(anyhow!("No layers?"))?;
    let check = layer.iter().filter(|&element| element == &1).count()
        * layer.iter().filter(|&element| element == &2).count();
    println!("Part 1: Checksum = {}", check);

    println!("Part 2: Image");
    println!("{}", image.render());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples_part2() {
        let image = Image::read((2, 2), Box::new("0222112222120000".as_bytes())).unwrap();

        assert_eq!(format!("{}", image.render()), " #\n# \n");
    }
}
