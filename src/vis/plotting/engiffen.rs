use color_quant::NeuQuant;
use fnv::FnvHashMap;
use gif::{Encoder, Frame, Repeat};
use lab::Lab;
use rayon::prelude::*;
use std::{borrow::Cow, error, f32, fmt, io, path::Path};

type Rgba = [u8; 4];

/// A color quantizing strategy.
///
/// `Naive` calculates color frequencies, picks the 256 most frequent colors
/// to be the palette, then reassigns the less frequently occuring colors to
/// the closest matching palette color.
///
/// `NeuQuant` uses the `NeuQuant` algorithm from the `color_quant` crate. It
/// trains a neural network using a pseudorandom subset of pixels, then
/// assigns each pixel its closest matching color in the palette.
///
/// # Usage
///
/// Pass this as the last argument to `engiffen` to select the quantizing
/// strategy.
///
/// The `NeuQuant` strategy produces the best looking images. Its interior
/// u32 value reduces the number of pixels that the algorithm uses to train,
/// which can greatly reduce its workload. Specifically, for a value of N,
/// only the pixels on every Nth column of every Nth row are considered, so
/// a value of 1 trains using every pixel, while a value of 2 trains using
/// 1/4 of all pixels.
///
/// The `Naive` strategy is fastest when you know that your input images
/// have a limited color range, but will produce terrible banding otherwise.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Quantizer {
    Naive,
    NeuQuant(u32),
}

/// An image, currently a wrapper around `image::DynamicImage`. If loaded from
/// disk through the `load_image` or `load_images` functions, its path property
/// contains the path used to read it from disk.
pub struct Image {
    pub pixels: Vec<Rgba>,
    pub width: u32,
    pub height: u32,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Image {{ dimensions: {} x {} }}",
            self.width, self.height
        )
    }
}

#[derive(Debug)]
pub enum Error {
    NoImages,
    Mismatch((u32, u32), (u32, u32)),
    ImageLoad(image::ImageError),
    ImageWrite(io::Error),
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Self::ImageLoad(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::ImageWrite(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NoImages => write!(f, "No frames sent for engiffening"),
            Self::Mismatch(_, _) => write!(f, "Frames don't have the same dimensions"),
            Self::ImageLoad(ref e) => write!(f, "Image load error: {e}"),
            Self::ImageWrite(ref e) => write!(f, "Image write error: {e}"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Self::NoImages => "No frames sent for engiffening",
            Self::Mismatch(_, _) => "Frames don't have the same dimensions",
            Self::ImageLoad(_) => "Unable to load image",
            Self::ImageWrite(_) => "Unable to write image",
        }
    }
}

/// Struct representing an animated Gif
#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Gif {
    pub palette: Vec<u8>,
    pub transparency: Option<u8>,
    pub width: u16,
    pub height: u16,
    pub images: Vec<Vec<u8>>,
    pub delay: u16,
}

impl fmt::Debug for Gif {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Gif {{ palette: Vec<u8 x {:?}>, transparency: {:?}, width: {:?}, height: {:?}, images: Vec<Vec<u8> x {:?}>, delay: {:?} }}",
            self.palette.len(),
            self.transparency,
            self.width,
            self.height,
            self.images.len(),
            self.delay
        )
    }
}

impl Gif {
    /// Writes the animated Gif to any output that implements Write.
    ///
    /// # Panics
    /// Panics if encoder could not be created
    ///
    /// # Errors
    ///
    /// Returns the `std::io::Result` of the underlying `write` function calls.
    pub fn write<W: io::Write>(&self, mut out: &mut W) -> Result<(), Error> {
        let mut encoder = Encoder::new(&mut out, self.width, self.height, &self.palette).unwrap();
        encoder.set_repeat(Repeat::Infinite).unwrap();
        for img in &self.images {
            let frame = Frame {
                delay: self.delay / 10,
                width: self.width,
                height: self.height,
                buffer: Cow::Borrowed(img),
                transparent: self.transparency,
                ..Default::default()
            };
            encoder.write_frame(&frame).unwrap();
        }
        Ok(())
    }
}

/// Loads an image from the given file path.
///
///
/// # Errors
///
/// Returns an error if the path can't be read or if the image can't be decoded
pub fn load_image<P>(path: P) -> Result<Image, Error>
where
    P: AsRef<Path>,
{
    let img = image::open(&path)?;
    let width = img.width();
    let height = img.height();
    let mut pixels: Vec<Rgba> = Vec::with_capacity(0);
    for (_, _, px) in img.into_rgba8().enumerate_pixels() {
        pixels.push(px.0);
    }
    Ok(Image {
        pixels,
        width,
        height,
    })
}

/// Loads images from a list of given paths. Errors encountered while loading files
/// are skipped.
///
/// Skips images that fail to load. If all images fail, returns an empty vector.
pub fn load_images<P>(paths: &[P]) -> Vec<Image>
where
    P: AsRef<Path>,
{
    paths
        .iter()
        .map(load_image)
        .filter_map(std::result::Result::ok)
        .collect()
}

/// Converts a sequence of images into a `Gif` at a given frame rate. The `quantizer`
/// parameter selects the algorithm that quantizes the palette into 256-colors.
///
/// # Panics
/// Panics if fps, height or width do not fit into a u16
///
/// # Errors
///
/// If any image dimensions differ, this function will return an `Error::Mismatch`
/// containing tuples of the conflicting image dimensions.
pub fn engiffen(imgs: &[Image], fps: usize, quantizer: Quantizer) -> Result<Gif, Error> {
    if imgs.is_empty() {
        return Err(Error::NoImages);
    }

    let (width, height) = {
        let first = &imgs[0];
        let first_dimensions = (first.width, first.height);
        for img in imgs {
            let other_dimensions = (img.width, img.height);
            if first_dimensions != other_dimensions {
                return Err(Error::Mismatch(first_dimensions, other_dimensions));
            }
        }
        first_dimensions
    };

    let (palette, palettized_imgs, transparency) = match quantizer {
        Quantizer::NeuQuant(sample_rate) => neuquant_palettize(imgs, sample_rate, width, height),
        Quantizer::Naive => naive_palettize(imgs),
    };

    let delay = u16::try_from(1000 / fps).unwrap();

    Ok(Gif {
        palette,
        transparency,
        width: u16::try_from(width).unwrap(),
        height: u16::try_from(height).unwrap(),
        images: palettized_imgs,
        delay,
    })
}

fn neuquant_palettize(
    imgs: &[Image],
    sample_rate: u32,
    width: u32,
    height: u32,
) -> (Vec<u8>, Vec<Vec<u8>>, Option<u8>) {
    let image_len = (width * height * 4 / sample_rate / sample_rate) as usize;
    let width = width as usize;
    let sample_rate = sample_rate as usize;
    let transparent_black = [0u8; 4];
    #[cfg(feature = "debug-stderr")]
    let time_push = Instant::now();
    let colors: Vec<u8> = imgs
        .par_iter()
        .map(|img| {
            let mut temp: Vec<_> = Vec::with_capacity(image_len);
            for (n, px) in img.pixels.iter().enumerate() {
                if sample_rate > 1 && (n % sample_rate != 0 || (n / width) % sample_rate != 0) {
                    continue;
                }
                if px[3] == 0 {
                    temp.extend_from_slice(&transparent_black);
                } else {
                    temp.extend_from_slice(&px[..3]);
                    temp.push(255);
                }
            }
            temp
        })
        .reduce(
            || Vec::with_capacity(image_len * imgs.len()),
            |mut acc, img| {
                acc.extend_from_slice(&img);
                acc
            },
        );

    let quant = NeuQuant::new(10, 256, &colors);

    let mut transparency = None;
    let mut cache: FnvHashMap<Rgba, u8> = FnvHashMap::default();
    let palettized_imgs: Vec<Vec<u8>> = imgs
        .iter()
        .map(|img| {
            img.pixels
                .iter()
                .map(|px| {
                    *cache.entry(*px).or_insert_with(|| {
                        let idx = u8::try_from(quant.index_of(px)).unwrap();
                        if transparency.is_none() && px[3] == 0 {
                            transparency = Some(idx);
                        }
                        idx
                    })
                })
                .collect()
        })
        .collect();

    (quant.color_map_rgb(), palettized_imgs, transparency)
}

fn naive_palettize(imgs: &[Image]) -> (Vec<u8>, Vec<Vec<u8>>, Option<u8>) {
    #[cfg(feature = "debug-stderr")]
    let time_count = Instant::now();
    let frequencies: FnvHashMap<Rgba, usize> = imgs
        .par_iter()
        .map(|img| {
            let mut fr: FnvHashMap<Rgba, usize> = FnvHashMap::default();
            for pixel in &img.pixels {
                let num = fr.entry(*pixel).or_insert(0);
                *num += 1;
            }
            fr
        })
        .reduce(FnvHashMap::default, |mut acc, fr| {
            for (color, count) in fr {
                let num = acc.entry(color).or_insert(0);
                *num += count;
            }
            acc
        });
    let mut sorted_frequencies = frequencies.into_iter().collect::<Vec<_>>();
    sorted_frequencies.sort_by(|a, b| b.1.cmp(&a.1));
    let sorted = sorted_frequencies
        .into_iter()
        .map(|c| (c.0, Lab::from_rgba(&c.0)))
        .collect::<Vec<_>>();

    let (palette, rest) = if sorted.len() > 256 {
        (&sorted[..256], &sorted[256..])
    } else {
        (&sorted[..], &[] as &[_])
    };

    let mut map: FnvHashMap<Rgba, u8> = FnvHashMap::default();
    for (i, color) in palette.iter().enumerate() {
        map.insert(color.0, u8::try_from(i).unwrap());
    }
    for color in rest {
        let closest_index = palette
            .iter()
            .enumerate()
            .fold((0, f32::INFINITY), |closest, (idx, p)| {
                let dist = p.1.squared_distance(&color.1);
                if closest.1 < dist {
                    closest
                } else {
                    (idx, dist)
                }
            })
            .0;
        let closest_rgb = palette[closest_index].0;
        let index = *map.get(&closest_rgb).expect(
            "A color we assigned to the palette is somehow missing from the palette index map.",
        );
        map.insert(color.0, index);
    }
    let palettized_imgs: Vec<Vec<u8>> = imgs
        .par_iter()
        .map(|img| {
            img.pixels
                .iter()
                .map(|px| {
                    *map.get(px)
                        .expect("A color in an image was not added to the palette map.")
                })
                .collect()
        })
        .collect();

    let mut palette_as_bytes = Vec::with_capacity(palette.len() * 3);
    for color in palette {
        palette_as_bytes.extend_from_slice(&color.0[0..3]);
    }

    (palette_as_bytes, palettized_imgs, None)
}
