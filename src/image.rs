use crate::commands::{Command, ImgFormat, Point, StreamImgFormat};
use crate::protocol;
use crate::traits::Serializable;
use log::*;

/// Contains an image
pub struct Image<'a> {
    pub width: u16,
    pub format: ImgFormat,
    pub data: &'a [u8],
    //pub coord: Point,
}

impl<'a> Image<'a> {}
