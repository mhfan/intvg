
//pub mod TinyVG {

use std::{io, num::TryFromIntError, fmt::{self, Display, Formatter}/*, marker::PhantomData*/};
//use byteorder::{ReadBytesExt, WriteBytesExt, LE};

#[derive(Debug)] pub enum ErrorKind {   IO(io::Error), IntError(TryFromIntError),
    InvalidData(u8), OutOfRange, //BadPosition, Fatal,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {  match self {
            ErrorKind::IO(e) => write!(f, "I/O error: {e}"),
            ErrorKind::IntError(e) =>
                write!(f, "Number conversion error: {e}"),
            ErrorKind::InvalidData(v) => write!(f, "Invalid data: {v:#x}"),
            ErrorKind::OutOfRange => write!(f, "Value out of range"),
        }
    }
}

type Result<T> = std::result::Result<T, TVGError>;
#[derive(Debug)] pub struct TVGError { kind: ErrorKind, msg: &'static str, }

impl Display for TVGError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }
}

impl std::error::Error for TVGError { }

//impl From<ErrorKind> for TVGError {
//    fn from(kind: ErrorKind) -> Self { Self { kind, msg: "" } }
//}

impl From<io::Error> for TVGError {
    fn from(e: io::Error) -> Self { Self { kind: ErrorKind::IO(e), msg: "" } }
}

impl From<TryFromIntError> for TVGError {
    fn from(e: TryFromIntError) -> Self { Self { kind: ErrorKind::IntError(e), msg: "" } }
}

/// **TinyVG** files are made up of a header, followed by a color lookup table
/// and a sequence of commands terminated by a _end of file_ command.
///
/// - All integers are assumed to be encoded in little-endian byte order
/// if not specified otherwise.
///
/// - The _Type_ fields have no padding bits in between. If a field
/// does not align to a byte boundary, the next field will be offset
/// into the byte by the current fields bit offset + bit size.
/// This means, that two consecutive fields A (u3) and B (u5) can be
/// extracted from the byte by using (byte & 0x7) >> 0 for A and
/// (byte & 0xF8) >> 3 for B.
///
/// - If not specified otherwise, all coordinates in TinyVG are absolute
/// coordinates, including path nodes and gradients.
///
/// - A lot of encoded integers are encoded off-by-one, thus mapping 0 to
/// 1, 1 to 2 and so on. This is done as encoding these integers as 0
/// would be equivalent to removing the element from the file.
/// Thus, this can be used to encode some more elements with less bytes.
/// If this is the case, this is signaled by the use of value+1.
///
/// https://tinyvg.tech/download/specification.txt, https://github.com/TinyVG/sdk,
/// https://github.com/lily-mara/tinyvg-rs, https://github.com/dataphract/tinyvg-rs

pub struct TinyVG<R: io::Read, W: io::Write> {
    pub header: Header,     // In-memory representation of a TinyVG file
    pub color_table:  Vec<RGBA8888>,    // colors used in this image
    pub commands: Vec<Command>,         // commands required to render this image
    pub trailer:  Vec<u8>,  // Remaining data after the TinyVG image ended (EOF).
    // Can be used for arbitrary metadata, it is not defined by the spec.

    write_range: fn(&mut W, i32) -> Result<()>,
     read_range: fn(&mut R) ->  io::Result<i32>,
    //_reader: PhantomData<R>, _writer: PhantomData<W>,
}

pub type TVGBuf<'a> = TinyVG<io::Cursor<&'a [u8]>, io::Cursor<Vec<u8>>>;
pub type TVGFile = TinyVG<io::BufReader<std::fs::File>, io::BufWriter<std::fs::File>>;
pub type TVGImage = TVGFile;

//impl<R: io::Read, W: io::Write> Default for Image<R, W> { fn default() -> Self { Self::new() } }

impl<R: io::Read, W: io::Write> TinyVG<R, W> { #[allow(clippy::new_without_default)]
    pub(crate) fn new() -> Self { Self {
            header: Header { //magic: TVG_MAGIC, version: TVG_VERSION,
                scale: 0, color_fmt: ColorEncoding::RGBA8888,
                coord_range: CoordinateRange::Default,
                width: 0, height: 0, //color_count: VarUInt(0),
            },  color_table: vec![], commands: vec![], trailer: vec![],

             read_range: Self::read_default, write_range: Self::write_default,
            //_reader: PhantomData, _writer: PhantomData,
    } }

    pub fn lookup_color(&self, idx: VarUInt) -> RGBA8888 { self.color_table[idx as usize] }
    pub fn push_color(&mut self, color: RGBA8888) -> VarUInt {
        if let Some(idx) = self.color_table.iter().position(|c|
            c.r == color.r && c.g == color.g && c.b == color.b && c.a == color.a) { idx as _
        } else { self.color_table.push(color);  self.color_table.len() as u32 - 1 }
    }

    pub fn load_data(reader: &mut R) -> Result<Self> {
        let mut tvgd = Self::new();

        let val = reader.read_u16_le()?;    if  val != TVG_MAGIC {
            return Err(TVGError { kind: ErrorKind::InvalidData(val as _),
                msg: "incorrect magic number" });
        }
        let val  = reader.read_u8()?;       if  val != TVG_VERSION {
            return Err(TVGError { kind: ErrorKind::InvalidData(val),
                msg: "incorrect version" });
        }

        let val = reader.read_u8()?;   tvgd.header.scale = val & 0x0F;
        // TODO: scale rendering by change tvgd.header.scale?

        tvgd.header.coord_range = match val >> 6 {
            0 => { //tvgd.write_range = Self::write_default;
                   //tvgd. read_range = Self:: read_default;
                                                            CoordinateRange::Default  }
            1 => { tvgd.write_range = Self::write_reduced;
                   tvgd. read_range = Self:: read_reduced;  CoordinateRange::Reduced  }
            2 => { tvgd.write_range = Self::write_enhanced;
                   tvgd. read_range = Self:: read_enhanced; CoordinateRange::Enhanced }
            x => return Err(TVGError { kind: ErrorKind::InvalidData(x),
                msg: "unsupported color encoding" })
        };  assert!(tvgd.commands.is_empty() && tvgd.color_table.is_empty());

        tvgd.header.width  = (tvgd.read_range)(reader)? as _;
        tvgd.header.height = (tvgd.read_range)(reader)? as _;
        let color_count = reader.read_var_uint()?;
        tvgd.color_table.reserve_exact(color_count as _);

        tvgd.header.color_fmt = ColorEncoding::RGBA8888;
        match (val >> 4) & 0x03 {   // XXX: unified to RGBA8888
            0 => for _ in 0..color_count { tvgd.color_table.push(RGBA8888 {
                    r: reader.read_u8()?, g: reader.read_u8()?,
                    b: reader.read_u8()?, a: reader.read_u8()?,
            })},

            1 => for _ in 0..color_count { let val = reader.read_u16_le()?;
                tvgd.color_table.push(RGBA8888 {   r: ((val & 0x001F) << 3) as _,
                    g: ((val & 0x07E0) >> 3) as _, b: ((val & 0xF800) >> 8) as _, a: 255,
            })},

            2 => for _ in 0..color_count { tvgd.color_table.push(RGBA8888 {
                    r: (reader.read_f32_le()? * 255.0 + 0.5) as _,
                    g: (reader.read_f32_le()? * 255.0 + 0.5) as _,
                    b: (reader.read_f32_le()? * 255.0 + 0.5) as _,
                    a: (reader.read_f32_le()? * 255.0 + 0.5) as _,
            })},

            x => return Err(TVGError { kind: ErrorKind::InvalidData(x),
                msg: "custom color encoding is not supported" }) //ColorEncoding::Custom,
        }

        loop {  let cmd = tvgd.read_command(reader)?;
            if let Command::EndOfDocument = cmd {
                reader.read_to_end(&mut tvgd.trailer)?; break
            }   tvgd.commands.push(cmd);
        }

        println!("{:?}, {} colors, {} cmds/paths", &tvgd.header,
            tvgd.color_table.len(), tvgd.commands.len());   Ok(tvgd)
    }

    fn read_command(&self, reader: &mut R) -> Result<Command> {
        let val = reader.read_u8()?;    let skind = val >> 6;

        Ok(match val & 0x3F {   0 => Command::EndOfDocument,    // command_index
            1 => Command::FillPolyg(self.read_fillcmd(skind, reader, Self::read_point)?),
            2 => Command::FillRects(self.read_fillcmd(skind, reader, Self::read_rect)?),

            3 => {  let count = reader.read_var_uint()? + 1;
                let fill= self.read_style(skind, reader)?;
                let coll = self.read_path(count as _, reader)?;
                Command::FillPath(FillCMD { fill, coll })
            }
            4 => Command::DrawLines(self.read_drawcmd(skind, reader, Self::read_line)?),
            5 => Command::DrawLoop (self.read_drawcmd(skind, reader, Self::read_point)?, false),
            6 => Command::DrawLoop (self.read_drawcmd(skind, reader, Self::read_point)?, true),

            7 => {  let count = reader.read_var_uint()? + 1;
                let line= self.read_style(skind, reader)?;
                let lwidth = self.read_unit(reader)?;
                let coll = self.read_path(count as _, reader)?;
                Command::DrawPath(DrawCMD { line, lwidth, coll })
            }
            8 => {  let res = self.read_outline(skind, reader,
                Self::read_point)?;     Command::OutlinePolyg(res.0, res.1)
            }
            9 => {  let res  = self.read_outline(skind, reader,
                Self::read_rect)?;      Command::OutlineRects(res.0, res.1)
            }

           10 => {  let val = reader.read_u8()?;
                let fill= self.read_style(skind, reader)?;
                let line= self.read_style(val >> 6, reader)?;
                let lwidth = self.read_unit(reader)?;
                let coll = self.read_path((val & 0x3F) as usize + 1, reader)?;
                Command::OutlinePath(fill, DrawCMD { line, lwidth, coll })
            }
            x => return Err(TVGError { kind: ErrorKind::InvalidData(x),
                    msg: "unrecognized command tag" })
        })
    }

    fn read_fillcmd<T>(&self, fill_kind: u8, reader: &mut R,
        read_fn: impl Fn(&Self, &mut R) -> Result<T>) -> Result<FillCMD<T>> {
        let count = reader.read_var_uint()? + 1;
        let fill = self.read_style(fill_kind, reader)?;
        let mut coll = Vec::with_capacity(count as _);
        for _ in 0..count { coll.push(read_fn(self, reader)?); }
        Ok(FillCMD { fill, coll })
    }

    fn read_drawcmd<T>(&self, line_kind: u8, reader: &mut R,
        read_fn: impl Fn(&Self, &mut R) -> Result<T>) -> Result<DrawCMD<T>> {
        let count = reader.read_var_uint()? + 1;
        let line = self.read_style(line_kind, reader)?;
        let lwidth = self.read_unit(reader)?;
        let mut coll = Vec::with_capacity(count as _);
        for _ in 0..count { coll.push(read_fn(self, reader)?); }
        Ok(DrawCMD { line, lwidth, coll })
    }

    fn read_outline<T>(&self, fill_kind: u8, reader: &mut R,
        read_fn: impl Fn(&Self, &mut R) -> Result<T>) -> Result<(Style, DrawCMD<T>)> {
        let (mut coll, val) = (vec![], reader.read_u8()?);
        let fill = self.read_style(fill_kind, reader)?;
        let line = self.read_style(val  >> 6, reader)?;
        let lwidth = self.read_unit(reader)?;
        for _ in 0..((val & 0x3F) + 1) { coll.push(read_fn(self, reader)?); }
        Ok((fill, DrawCMD { line, lwidth, coll }))
    }

    fn read_path(&self, count: usize, reader: &mut R) -> Result<Vec<Segment>> {
        let mut vlen = Vec::with_capacity(count);
        let mut coll = Vec::with_capacity(count);
        for _ in 0..count { vlen.push(reader.read_var_uint()? + 1); }
        for len in vlen { coll.push(self.read_segment(len, reader)?); }
        Ok(coll)
    }

    fn read_segment(&self, len: u32, reader: &mut R) -> Result<Segment> {
        let mut cmds = Vec::with_capacity(len as _);
        let start = self.read_point(reader)?;
        for _ in 0..len {   let val = reader.read_u8()?;
            let lwidth = if 0 < val & 0x10 {
                Some(self.read_unit(reader)?) } else { None };

            let instr = match val & 0x07 {
                0 => SegInstr::Line  { end: self.read_point(reader)? },
                1 => SegInstr::HLine {   x: self.read_unit (reader)? },
                2 => SegInstr::VLine {   y: self.read_unit (reader)? },
                3 => SegInstr::CubicBezier { ctrl: (self.read_point(reader)?,
                        self.read_point(reader)?), end: self.read_point(reader)? },

                4 => {  let val = reader.read_u8()?;    SegInstr::ArcCircle {
                        large: 0 < val & 0x01, sweep: 0 < val & 0x02,
                        radius:   self.read_unit(reader)?, target: self.read_point(reader)?
                } }
                5 => {  let val = reader.read_u8()?;    SegInstr::ArcEllipse {
                        large: 0 < val & 0x01, sweep: 0 < val & 0x02,
                        radius:  (self.read_unit(reader)?, self.read_unit(reader)?),
                        rotation: self.read_unit(reader)?, target: self.read_point(reader)?
                } }

                6 => SegInstr::ClosePath,
                7 => SegInstr::QuadBezier {
                        ctrl: self.read_point(reader)?, end: self.read_point(reader)? },
                x => return Err(TVGError { kind: ErrorKind::InvalidData(x),
                        msg: "illegal path segment instruction" })
            };  cmds.push(SegmentCommand { instr, lwidth, });
        }   Ok(Segment { start, cmds })
    }

    fn read_style(&self, kind: u8, reader: &mut R) -> Result<Style> {
        Ok(match kind {
            0 =>   Style::FlatColor(reader.read_var_uint()?),
            1 => { Style::LinearGradient {
                    points: (self.read_point(reader)?, self.read_point(reader)?),
                    cindex: (reader.read_var_uint()?, reader.read_var_uint()?),
            } }
            2 => { Style::RadialGradient {
                    points: (self.read_point(reader)?, self.read_point(reader)?),
                    cindex: (reader.read_var_uint()?, reader.read_var_uint()?),
            } }
            x => return Err(TVGError { kind: ErrorKind::InvalidData(x),
                    msg: "unsupported primary style" })
        })
    }

    fn read_line(&self, reader: &mut R) -> Result<Line> {
        Ok(Line { start: self.read_point(reader)?, end: self.read_point(reader)? })
    }

    fn read_rect(&self, reader: &mut R) -> Result<Rect> {
        Ok(Rect { x: self.read_unit(reader)?, y: self.read_unit(reader)?,
                  w: self.read_unit(reader)?, h: self.read_unit(reader)? })
    }

    fn read_point(&self, reader: &mut R) -> Result<Point> {
        Ok(Point { x: self.read_unit(reader)?, y: self.read_unit(reader)? })
    }

    #[inline] fn read_default (reader: &mut R) ->
        io::Result<i32> { reader.read_u16_le().map(|v| i32::from(v as i16)) }
    #[inline] fn read_reduced (reader: &mut R) ->
        io::Result<i32> { reader.read_u8().map(|v| i32::from(v as i8)) }
    #[inline] fn read_enhanced(reader: &mut R) -> io::Result<i32> { reader.read_i32_le() }

    /** ```
    assert!(i16::MAX as u16 == 0x7fff && i16::MIN as u16 == 0x8000);
    assert!(0x8000_u16 as i16 == i16::MIN && 0xffff_u16 as i16 == -1);
    ``` */
    fn read_unit(&self, reader: &mut R) -> Result<Unit> {
        Ok((self.read_range)(reader)? as f32 / (1u32 << self.header.scale) as f32)
    }

    pub fn save_data(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(TVG_MAGIC)?;    writer.write_u8(TVG_VERSION)?;
        writer.write_u8((self.header.coord_range as u8) << 6 |
                        (self.header.color_fmt   as u8) << 4 | self.header.scale)?;

        (self.write_range)(writer, self.header.width  as _)?;
        (self.write_range)(writer, self.header.height as _)?;
        writer.write_var_uint(self.color_table.len()  as _)?;

        match self.header.color_fmt {  //ColorEncoding::Custom => (),
            ColorEncoding::RGBA8888 => for color in &self.color_table {
                    writer.write_u8(color.r)?; writer.write_u8(color.g)?;
                    writer.write_u8(color.b)?; writer.write_u8(color.a)?;
            },
            ColorEncoding::RGB565 => for color in &self.color_table {
                    writer.write_u16_le((color.r as u16  >> 3) |
                        ((color.g as u16) << 3) | ((color.b as u16) << 8))?;
            },
            ColorEncoding::RGBAf32 => for color in &self.color_table {
                    writer.write_f32_le(color.r as f32 / 255.0)?;
                    writer.write_f32_le(color.g as f32 / 255.0)?;
                    writer.write_f32_le(color.b as f32 / 255.0)?;
                    writer.write_f32_le(color.a as f32 / 255.0)?;
            },
        }

        self.commands.iter().try_for_each(|cmd| self.write_command(cmd, writer))?;
        writer.write_u8(0)?;    // Command::EndOfDocument

        Ok(writer.write_all(&self.trailer)?)
    }

    fn write_command(&self, cmd: &Command, writer: &mut W)-> Result<()> {
        match cmd {     Command::EndOfDocument => Ok(()),
            Command::FillPolyg(cmd) =>
                self.write_fillcmd(1, cmd, writer, Self::write_point),
            Command::FillRects(cmd) =>
                self.write_fillcmd(2, cmd, writer, Self::write_rect),

            Command::FillPath(cmd) => {
                writer.write_u8((cmd.fill.to_u8() << 6) | 3)?;
                writer.write_var_uint(cmd.coll.len() as u32 - 1)?;
                self.write_style(&cmd.fill, writer)?;
                self.write_path (&cmd.coll, writer)
            }

            Command::DrawLines(cmd) =>
                self.write_drawcmd(4, cmd, writer, Self::write_line),
            Command::DrawLoop (cmd, strip) => if *strip {
                self.write_drawcmd(6, cmd, writer, Self::write_point) } else {
                self.write_drawcmd(5, cmd, writer, Self::write_point) },

            Command::DrawPath(cmd) => {
                writer.write_u8((cmd.line.to_u8() << 6) | 7)?;
                writer.write_var_uint(cmd.coll.len() as u32 - 1)?;
                self.write_style(&cmd.line, writer)?;
                self.write_unit(cmd.lwidth, writer)?;
                self.write_path (&cmd.coll, writer)
            }

            Command::OutlinePolyg(fill, cmd) =>
                self.write_outline(8, fill, cmd, writer, Self::write_point),
            Command::OutlineRects(fill, cmd) =>
                self.write_outline(9, fill, cmd, writer, Self::write_rect),

            Command::OutlinePath (fill, cmd) => {
                writer.write_u8( (fill.to_u8() << 6) | 10)?;
                if (1 << 6) < cmd.coll.len() { return Err(TVGError {
                    kind: ErrorKind::OutOfRange, msg: "outline path segment" }) }
                writer.write_u8((cmd.line.to_u8() << 6) | (cmd.coll.len() as u8 - 1))?;
                self.write_style( fill, writer)?;       self.write_style(&cmd.line, writer)?;
                self.write_unit(cmd.lwidth, writer)?;   self.write_path (&cmd.coll, writer)
            }
        }
    }

    fn write_fillcmd<T>(&self, idx: u8, cmd: &FillCMD<T>, writer: &mut W,
        write_fn: impl Fn(&Self, &T, &mut W) -> Result<()>) -> Result<()> {
        writer.write_u8((cmd.fill.to_u8() << 6) | idx)?;
        writer.write_var_uint(cmd.coll.len() as u32 - 1)?;
        self.write_style(&cmd.fill, writer)?;
        cmd.coll.iter().try_for_each(|elem| write_fn(self, elem, writer))
    }

    fn write_drawcmd<T>(&self, idx: u8, cmd: &DrawCMD<T>, writer: &mut W,
        write_fn: impl Fn(&Self, &T, &mut W) -> Result<()>) -> Result<()> {
        writer.write_u8((cmd.line.to_u8() << 6) | idx)?;
        writer.write_var_uint(cmd.coll.len() as u32 - 1)?;
        self.write_style(&cmd.line, writer)?;   self.write_unit(cmd.lwidth, writer)?;
        cmd.coll.iter().try_for_each(|elem| write_fn(self, elem, writer))
    }

    fn write_outline<T>(&self, idx: u8, fill: &Style, cmd: &DrawCMD<T>, writer: &mut W,
        write_fn: impl Fn(&Self, &T, &mut W) -> Result<()>) -> Result<()> {
        writer.write_u8((fill.to_u8() << 6) | idx)?;
        if (1 << 6) < cmd.coll.len() { return Err(TVGError {
            kind: ErrorKind::OutOfRange, msg: "outline path segment" }) }
        writer.write_u8((cmd.line.to_u8() << 6) | (cmd.coll.len() as u8 - 1))?;
        self.write_style(fill, writer)?;        self.write_style(&cmd.line, writer)?;
        self.write_unit(cmd.lwidth, writer)?;
        cmd.coll.iter().try_for_each(|elem| write_fn(self, elem, writer))
    }

    fn write_path(&self, coll: &Vec<Segment>, writer: &mut W) -> Result<()> {
        for seg in coll { writer.write_var_uint(seg.cmds.len() as u32 - 1)? }
        coll.iter().try_for_each(|seg| self.write_segment(seg, writer))
    }

    fn write_segment(&self, seg: &Segment, writer: &mut W) -> Result<()> {
        self.write_point(&seg.start, writer)?;
        seg.cmds.iter().try_for_each(|cmd| {
            let mut write_tag = |idx| {
                if let Some(val) = cmd.lwidth {
                    writer.write_u8(idx | 0x10)?;   self.write_unit(val, writer)
                } else { Ok(writer.write_u8(idx)?) }
            };

            match &cmd.instr {
                SegInstr::Line { end } => { write_tag(0)?; self.write_point(end, writer) }
                SegInstr::HLine { x } => {    write_tag(1)?; self.write_unit(*x, writer) }
                SegInstr::VLine { y } => {    write_tag(2)?; self.write_unit(*y, writer) }
                SegInstr::CubicBezier {
                    ctrl, end } => {  write_tag(3)?;
                    self.write_point(&ctrl.0, writer)?;
                    self.write_point(&ctrl.1, writer)?;
                    self.write_point(end, writer)
                }
                SegInstr::ArcCircle { large, sweep,
                    radius, target } => {   write_tag(4)?;
                    let mut val = 0u8;  if *large { val |= 0x01; }
                    if *sweep { val |= 0x02; }  writer.write_u8(val)?;
                    self.write_unit(*radius, writer)?;      self.write_point(target, writer)
                }
                SegInstr::ArcEllipse { large, sweep, radius,
                    rotation, target } => {     write_tag(5)?;
                    let mut val = 0u8;  if *large { val |= 0x01; }
                    if *sweep { val |= 0x02; }  writer.write_u8(val)?;
                    self.write_unit(radius.0, writer)?;     self.write_unit(radius.1, writer)?;
                    self.write_unit(*rotation, writer)?;    self.write_point(target, writer)
                }
                SegInstr::ClosePath => write_tag(6),
                SegInstr::QuadBezier { ctrl, end } => {  write_tag(7)?;
                    self.write_point(ctrl, writer)?;     self.write_point(end, writer)
                }
            }
        })
    }

    fn write_style(&self, style: &Style, writer: &mut W) -> Result<()> {
        let mut write_gradient =
            |points: &(Point, Point), cindex: &(VarUInt, VarUInt)| {
            if cindex.0 >= self.color_table.len() as u32 ||
               cindex.1 >= self.color_table.len() as u32 { return Err(TVGError {
                kind: ErrorKind::OutOfRange, msg: "invalid color index" }) }
            self.write_point(&points.0, writer)?;   self.write_point(&points.1, writer)?;
            writer.write_var_uint(cindex.0)?;  Ok(writer.write_var_uint(cindex.1)?)
        };

        match style {
            Style::FlatColor(idx) => {
                if *idx >= self.color_table.len() as u32 { return Err(TVGError {
                    kind: ErrorKind::OutOfRange, msg: "invalid color index" }) }
                Ok(writer.write_var_uint(*idx)?)
            }
            Style::LinearGradient { points, cindex } =>
                write_gradient(points, cindex),
            Style::RadialGradient { points, cindex } =>
                write_gradient(points, cindex),
        }
    }

    fn write_line(&self, line: &Line, writer: &mut W) -> Result<()> {
        self.write_point(&line.start, writer)?; self.write_point(&line.end, writer)
    }

    fn write_rect(&self, rect: &Rect, writer: &mut W) -> Result<()> {
        self.write_unit(rect.x, writer)?;   self.write_unit(rect.y, writer)?;
        self.write_unit(rect.w, writer)?;   self.write_unit(rect.h, writer)
    }

    fn write_point(&self, point: &Point, writer: &mut W) -> Result<()> {
        self.write_unit(point.x, writer)?;  self.write_unit(point.y, writer)
    }

    #[inline] fn write_default (writer: &mut W, val: i32) ->
        Result<()> { Ok(writer.write_u16_le(i16::try_from(val)? as u16)?) }
    #[inline] fn write_reduced (writer: &mut W, val: i32) ->
        Result<()> { Ok(writer.write_u8(i8::try_from(val)? as u8)?) }
    #[inline] fn write_enhanced(writer: &mut W, val: i32) ->
        Result<()> { Ok(writer.write_i32_le(val)?) }

    fn write_unit(&self, val: Unit, writer: &mut W)-> Result<()> {
        (self.write_range)(writer, (val * (1u32 << self.header.scale) as f32 + 0.5) as i32)
    }
}

const TVG_MAGIC: u16  = 0x5672; // [0x72, 0x56];
const TVG_VERSION: u8 = 1;

#[derive(Debug)] pub struct Header {
    //magic: u16,     // Must be [0x72, 0x56], 0x5672
    //version: u8,    // Must be 1. This field might decide how the rest of the format looks like.

    pub scale: u8,  // u4, Defines the number of fraction bits in a _Unit_ value.

    /// u2, Defines the type of color information that is used in the _color table_.
    pub color_fmt: ColorEncoding,

    /// u2, Defines the number of total bits in a _Unit_ value
    /// and thus the overall precision of the file.
    pub coord_range: CoordinateRange,

    /// Encodes the maximum width/height of the output file in _display units_.
    /// A value of `0` indicates that the image has the maximum possible width.
    /// The size of these two fields depends on the _coordinate range_ field.
    pub width: u32, pub height: u32,    // u8, u16 or u32

    //color_count: VarUInt,   // The number of colors in the _color table_.
}

/// **VarUInt**:
/// This type is used to encode 32 bits unsigned integers while keeping the number
/// of bytes low. It is encoded as a variable-sized integer that uses 7 bits per byte
/// for integer bits and the 7th bits to encode that there is "more bits available".
///
/// The integer is still built as a little-endian, so the first byte will always encode
/// bits 0…6, the second one encodes 8…13, and so on.
/// Bytes are read until the uppermost bit in the byte is not set.
///
/// The bit mappings are done as following:
///
/// Byte | Bit Range | Notes
/// ---- | --------- | ------------------------------------------------------------------
/// #1   | 0…6       | This byte must always be present.
/// #2   | 7…13      |
/// #3   | 14…20     |
/// #4   | 21…27     |
/// #5   | 28…31     | This byte must always have the uppermost 4 bits set as 0b0000????.
///
/// So a VarUInt always has between 1 and 5 bytes while mapping the full range of
/// a 32 bits value. This means we only have 5 bits overhead in the worst case, but for
/// all smaller values, we reduce the number of bytes for encoding unsigned integers.
//#[derive(Clone, Copy)] struct VarUInt(u32);
type VarUInt = u32;

//#[derive(Clone, Copy)] struct Unit(f32);
type Unit = f32;    // Each Unit takes up 16/8/32 bits, // XXX: can be fixed-point?

trait TVGRead: io::Read  {
    /*#[inline] fn read_value<T>(&mut self) -> io::Result<T> {    // XXX:
        let mut buf = [0; core::mem::size_of::<T>()];
        self.read_exact(&mut buf)?; Ok(T::from_le_bytes(buf))
    }*/

    #[inline] fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1]; self.read_exact(&mut buf)?; Ok(buf[0]) }
    #[inline] fn read_u16_le(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2]; self.read_exact(&mut buf)?; Ok(u16::from_le_bytes(buf)) }
    #[inline] fn read_i32_le(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4]; self.read_exact(&mut buf)?; Ok(i32::from_le_bytes(buf)) }
    #[inline] fn read_f32_le(&mut self) -> io::Result<f32> {    // read_f32::<LE>()
        let mut buf = [0; 4]; self.read_exact(&mut buf)?; Ok(f32::from_le_bytes(buf)) }

    fn  read_var_uint(&mut self) -> Result<VarUInt> {
        let mut val = 0u32;
        for cnt in 0..5 {
            let tmp = self.read_u8()?;
            val |= ((tmp & 0x7F) as u32) << (7 * cnt);
            if tmp < 0x80 { return Ok(val); }
        }

        Err(TVGError { kind: ErrorKind::InvalidData(val as _),
            msg: "Invalid VarUInt encoding; too many bytes" })
    }
}

trait TVGWrite: io::Write {
    /*#[inline] fn write_value<T>(&mut self, n: T) ->
        io::Result<()> { self.write_all(&n.to_le_bytes()) }
    }*/

    #[inline] fn write_u8(&mut self, n: u8) -> io::Result<()> { self.write_all(&[n]) }
    #[inline] fn write_u16_le(&mut self, n: u16) ->     // write_u16::<LE>(n)
        io::Result<()> { self.write_all(&n.to_le_bytes()) }
    #[inline] fn write_i32_le(&mut self, n: i32) ->
        io::Result<()> { self.write_all(&n.to_le_bytes()) }
    #[inline] fn write_f32_le(&mut self, n: f32) ->
        io::Result<()> { self.write_all(&n.to_le_bytes()) }

    fn write_var_uint(&mut self, mut val: u32)-> io::Result<()> {
        loop {  let (flag, tmp) = (val < 0x80, val as u8);
            if flag { self.write_u8(tmp)?;  break
            }  else { self.write_u8(tmp | 0x80)?; val >>= 7; }
        }   Ok(())
    }
}

impl<W: io::Write> TVGWrite for W {}
impl<R: io::Read>  TVGRead  for R {}

#[derive(Debug, Clone, Copy)] pub enum CoordinateRange { Default = 0, Reduced = 1, Enhanced = 2 }
#[derive(Debug, Clone, Copy)] pub enum ColorEncoding { RGBA8888 = 0, RGB565 = 1, RGBAf32 = 2, }

//#[derive(Clone, Copy)] struct RGB565(u16);     // sRGB color space
#[derive(Clone, Copy)] pub struct RGBA8888 { pub r:  u8, pub g:  u8, pub b:  u8, pub a:  u8 }
//struct RGBAf32  { r: f32, g: f32, b: f32, a: f32 }  // scRGB color space
// color channel between 0 and 100% intensity, mapped to value range
//use tiny_skia::{ColorU8, Color, Rect, Point};     // XXX: tiny_skia_path

/// **Commands**:
/// TinyVG files contain a sequence of draw commands that must be executed in the defined
/// order to get the final result. Each draw command adds a new 2D primitive to the graphic.
///
/// Each command is encoded as a single byte which is split into fields:
///
/// Field           | Type | Description
/// --------------- | ---- | -------------------------------------------------------
/// command_index   | u6   | The command that is encoded next. See table above.
/// prim_style_kind | u2   | The type of style this command uses as a primary style.
pub enum Command { EndOfDocument,
    FillPolyg(FillCMD<Point>), FillRects(FillCMD<Rect>), FillPath(FillCMD<Segment>),
    DrawLines(DrawCMD<Line>),  DrawLoop (DrawCMD<Point>, bool), //DrawStrip(DrawCMD<Point>),
    DrawPath (DrawCMD<Segment>),        OutlinePolyg(Style, DrawCMD<Point>),
    OutlineRects(Style, DrawCMD<Rect>), OutlinePath (Style, DrawCMD<Segment>),
}

pub struct FillCMD<T> { pub fill: Style, pub coll: Vec<T> }     // line -> stroke
/// Each line is line_width units wide, and at least a single display pixel.
/// This means that line_width of 0 is still visible, even though only marginally.
pub struct DrawCMD<T> { pub line: Style, pub lwidth: Unit, pub coll: Vec<T> }

pub enum Style { FlatColor(VarUInt),   // color_index in the color_table
    LinearGradient { points: (Point, Point), cindex: (VarUInt, VarUInt), },
    /// The gradient is formed by a mental circle with the center at point_0 and
    /// point_1 being somewhere on the circle outline. Thus, the radius of said
    /// circle is the distance between point_0 and point_1.
    RadialGradient { points: (Point, Point), cindex: (VarUInt, VarUInt), },
}

impl Style {
    #[inline] fn to_u8(&self) -> u8 { match self { Self::FlatColor(_) => 0,
            Self::LinearGradient {..} => 1, Self::RadialGradient {..} => 2, }
    }
}

pub struct Line { pub start: Point, pub end: Point, }

/// **Point**: Points are a X and Y coordinate pair.
///
/// Field | Type   | Description
/// ----- | ------ | -----------------------------------------------
/// x     | _Unit_ | Horizontal distance of the point to the origin.
/// y     | _Unit_ | Vertical   distance of the point to the origin.
///
/// **Units**:
/// The unit is the common type for both positions and sizes in the vector graphic.
/// It is encoded as a signed integer with a configurable amount of bits
/// (see _Coordinate Range_) and fractional bits.
///
/// The file header defines a _scale_ by which each signed integer is divided into
/// the final value. For example, with a _reduced_ value of 0x13 and a scale of 4,
/// we get the final value of 1.1875, as the number is interpretet as binary b0001.0011.
#[derive(Clone, Copy)] pub struct Point { pub x: Unit, pub y: Unit }
#[derive(Clone, Copy)] pub struct Rect  { pub x: Unit, pub y: Unit, pub w: Unit, pub h: Unit }

/// **Paths** describe instructions to create complex 2D graphics.
///
/// The mental model to form the path is this:
/// Each path segment generates a shape by moving a "pen" around. The path this "pen" takes
/// is the outline of our segment. Each segment, the "pen" starts at a defined position and
/// is moved by instructions. Each instruction will leave the "pen" at a new position.
/// The line drawn by our "pen" is the outline of the shape.
///
/// 1.  For each segment in the path, the number of commands is encoded as a VarUInt-1.
///     Decoding a 0 means that 1 element is stored in the segment.
///
/// 2.  For each segment in the path:
///
///     1) A Point is encoded as the starting point.
///
///     2) The instructions for this path, the number is determined in the first step.
///
///     3) Each instruction is prefixed by a single tag byte that encodes the kind of
///        instruction as well as the information if a line width is present.
///
///     4) If a line width is present, that line width is read as a Unit
///
///     5) The data for this command is decoded.
pub struct Segment { pub start: Point, pub cmds: Vec<SegmentCommand>, }

pub struct SegmentCommand { pub instr: SegInstr, pub lwidth: Option<Unit>, }

pub enum SegInstr { //Move { start: Point },
    Line { end: Point, }, HLine { x: Unit, }, VLine { y: Unit, },
    CubicBezier { ctrl: (Point, Point), end: Point, },
    ArcCircle  { large: bool, sweep: bool, radius:  Unit, target: Point, },
    ArcEllipse { large: bool, sweep: bool, radius: (Unit, Unit), rotation: Unit, target: Point, },
    QuadBezier { ctrl: Point, end: Point, },     ClosePath,
}

//}

