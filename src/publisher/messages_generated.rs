// automatically generated by the FlatBuffers compiler, do not modify



use std::mem;
use std::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(unused_imports, dead_code)]
pub mod switchboard {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Content {
  NONE = 0,
  ViewStart = 1,
  ViewUpdate = 2,
  ViewAck = 3,
  ViewEnd = 4,
  Broadcast = 5,

}

const ENUM_MIN_CONTENT: u8 = 0;
const ENUM_MAX_CONTENT: u8 = 5;

impl<'a> flatbuffers::Follow<'a> for Content {
  type Inner = Self;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    flatbuffers::read_scalar_at::<Self>(buf, loc)
  }
}

impl flatbuffers::EndianScalar for Content {
  #[inline]
  fn to_little_endian(self) -> Self {
    let n = u8::to_le(self as u8);
    let p = &n as *const u8 as *const Content;
    unsafe { *p }
  }
  #[inline]
  fn from_little_endian(self) -> Self {
    let n = u8::from_le(self as u8);
    let p = &n as *const u8 as *const Content;
    unsafe { *p }
  }
}

impl flatbuffers::Push for Content {
    type Output = Content;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        flatbuffers::emplace_scalar::<Content>(dst, *self);
    }
}

#[allow(non_camel_case_types)]
const ENUM_VALUES_CONTENT:[Content; 6] = [
  Content::NONE,
  Content::ViewStart,
  Content::ViewUpdate,
  Content::ViewAck,
  Content::ViewEnd,
  Content::Broadcast
];

#[allow(non_camel_case_types)]
const ENUM_NAMES_CONTENT:[&'static str; 6] = [
    "NONE",
    "ViewStart",
    "ViewUpdate",
    "ViewAck",
    "ViewEnd",
    "Broadcast"
];

pub fn enum_name_content(e: Content) -> &'static str {
  let index = e as u8;
  ENUM_NAMES_CONTENT[index as usize]
}

pub struct ContentUnionTableOffset {}
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Capability {
  FullScreen = 0,
  BitwiseIncremental = 1,

}

const ENUM_MIN_CAPABILITY: u32 = 0;
const ENUM_MAX_CAPABILITY: u32 = 1;

impl<'a> flatbuffers::Follow<'a> for Capability {
  type Inner = Self;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    flatbuffers::read_scalar_at::<Self>(buf, loc)
  }
}

impl flatbuffers::EndianScalar for Capability {
  #[inline]
  fn to_little_endian(self) -> Self {
    let n = u32::to_le(self as u32);
    let p = &n as *const u32 as *const Capability;
    unsafe { *p }
  }
  #[inline]
  fn from_little_endian(self) -> Self {
    let n = u32::from_le(self as u32);
    let p = &n as *const u32 as *const Capability;
    unsafe { *p }
  }
}

impl flatbuffers::Push for Capability {
    type Output = Capability;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        flatbuffers::emplace_scalar::<Capability>(dst, *self);
    }
}

#[allow(non_camel_case_types)]
const ENUM_VALUES_CAPABILITY:[Capability; 2] = [
  Capability::FullScreen,
  Capability::BitwiseIncremental
];

#[allow(non_camel_case_types)]
const ENUM_NAMES_CAPABILITY:[&'static str; 2] = [
    "FullScreen",
    "BitwiseIncremental"
];

pub fn enum_name_capability(e: Capability) -> &'static str {
  let index = e as u32;
  ENUM_NAMES_CAPABILITY[index as usize]
}

pub enum BroadcastOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct Broadcast<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Broadcast<'a> {
    type Inner = Broadcast<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> Broadcast<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Broadcast {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args BroadcastArgs<'args>) -> flatbuffers::WIPOffset<Broadcast<'bldr>> {
      let mut builder = BroadcastBuilder::new(_fbb);
      if let Some(x) = args.text { builder.add_text(x); }
      builder.finish()
    }

    pub const VT_TEXT: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn text(&self) -> Option<&'a str> {
    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(Broadcast::VT_TEXT, None)
  }
}

pub struct BroadcastArgs<'a> {
    pub text: Option<flatbuffers::WIPOffset<&'a  str>>,
}
impl<'a> Default for BroadcastArgs<'a> {
    #[inline]
    fn default() -> Self {
        BroadcastArgs {
            text: None,
        }
    }
}
pub struct BroadcastBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> BroadcastBuilder<'a, 'b> {
  #[inline]
  pub fn add_text(&mut self, text: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Broadcast::VT_TEXT, text);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> BroadcastBuilder<'a, 'b> {
    let start = _fbb.start_table();
    BroadcastBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Broadcast<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum ViewStartOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct ViewStart<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ViewStart<'a> {
    type Inner = ViewStart<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> ViewStart<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ViewStart {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ViewStartArgs) -> flatbuffers::WIPOffset<ViewStart<'bldr>> {
      let mut builder = ViewStartBuilder::new(_fbb);
      builder.add_capabilities(args.capabilities);
      builder.finish()
    }

    pub const VT_CAPABILITIES: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn capabilities(&self) -> u32 {
    self._tab.get::<u32>(ViewStart::VT_CAPABILITIES, Some(0)).unwrap()
  }
}

pub struct ViewStartArgs {
    pub capabilities: u32,
}
impl<'a> Default for ViewStartArgs {
    #[inline]
    fn default() -> Self {
        ViewStartArgs {
            capabilities: 0,
        }
    }
}
pub struct ViewStartBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ViewStartBuilder<'a, 'b> {
  #[inline]
  pub fn add_capabilities(&mut self, capabilities: u32) {
    self.fbb_.push_slot::<u32>(ViewStart::VT_CAPABILITIES, capabilities, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ViewStartBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ViewStartBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ViewStart<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum ViewEndOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct ViewEnd<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ViewEnd<'a> {
    type Inner = ViewEnd<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> ViewEnd<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ViewEnd {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        _args: &'args ViewEndArgs) -> flatbuffers::WIPOffset<ViewEnd<'bldr>> {
      let mut builder = ViewEndBuilder::new(_fbb);
      builder.finish()
    }

}

pub struct ViewEndArgs {
}
impl<'a> Default for ViewEndArgs {
    #[inline]
    fn default() -> Self {
        ViewEndArgs {
        }
    }
}
pub struct ViewEndBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ViewEndBuilder<'a, 'b> {
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ViewEndBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ViewEndBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ViewEnd<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum ViewAckOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct ViewAck<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ViewAck<'a> {
    type Inner = ViewAck<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> ViewAck<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ViewAck {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ViewAckArgs) -> flatbuffers::WIPOffset<ViewAck<'bldr>> {
      let mut builder = ViewAckBuilder::new(_fbb);
      builder.add_sqn(args.sqn);
      builder.finish()
    }

    pub const VT_SQN: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn sqn(&self) -> u32 {
    self._tab.get::<u32>(ViewAck::VT_SQN, Some(0)).unwrap()
  }
}

pub struct ViewAckArgs {
    pub sqn: u32,
}
impl<'a> Default for ViewAckArgs {
    #[inline]
    fn default() -> Self {
        ViewAckArgs {
            sqn: 0,
        }
    }
}
pub struct ViewAckBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ViewAckBuilder<'a, 'b> {
  #[inline]
  pub fn add_sqn(&mut self, sqn: u32) {
    self.fbb_.push_slot::<u32>(ViewAck::VT_SQN, sqn, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ViewAckBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ViewAckBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ViewAck<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum TileOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct Tile<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Tile<'a> {
    type Inner = Tile<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> Tile<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Tile {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args TileArgs<'args>) -> flatbuffers::WIPOffset<Tile<'bldr>> {
      let mut builder = TileBuilder::new(_fbb);
      if let Some(x) = args.data { builder.add_data(x); }
      builder.add_h(args.h);
      builder.add_w(args.w);
      builder.add_y(args.y);
      builder.add_x(args.x);
      builder.finish()
    }

    pub const VT_X: flatbuffers::VOffsetT = 4;
    pub const VT_Y: flatbuffers::VOffsetT = 6;
    pub const VT_W: flatbuffers::VOffsetT = 8;
    pub const VT_H: flatbuffers::VOffsetT = 10;
    pub const VT_DATA: flatbuffers::VOffsetT = 12;

  #[inline]
  pub fn x(&self) -> u16 {
    self._tab.get::<u16>(Tile::VT_X, Some(0)).unwrap()
  }
  #[inline]
  pub fn y(&self) -> u16 {
    self._tab.get::<u16>(Tile::VT_Y, Some(0)).unwrap()
  }
  #[inline]
  pub fn w(&self) -> u16 {
    self._tab.get::<u16>(Tile::VT_W, Some(0)).unwrap()
  }
  #[inline]
  pub fn h(&self) -> u16 {
    self._tab.get::<u16>(Tile::VT_H, Some(0)).unwrap()
  }
  #[inline]
  pub fn data(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(Tile::VT_DATA, None).map(|v| v.safe_slice())
  }
}

pub struct TileArgs<'a> {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub data: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a ,  u8>>>,
}
impl<'a> Default for TileArgs<'a> {
    #[inline]
    fn default() -> Self {
        TileArgs {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            data: None,
        }
    }
}
pub struct TileBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> TileBuilder<'a, 'b> {
  #[inline]
  pub fn add_x(&mut self, x: u16) {
    self.fbb_.push_slot::<u16>(Tile::VT_X, x, 0);
  }
  #[inline]
  pub fn add_y(&mut self, y: u16) {
    self.fbb_.push_slot::<u16>(Tile::VT_Y, y, 0);
  }
  #[inline]
  pub fn add_w(&mut self, w: u16) {
    self.fbb_.push_slot::<u16>(Tile::VT_W, w, 0);
  }
  #[inline]
  pub fn add_h(&mut self, h: u16) {
    self.fbb_.push_slot::<u16>(Tile::VT_H, h, 0);
  }
  #[inline]
  pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Tile::VT_DATA, data);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> TileBuilder<'a, 'b> {
    let start = _fbb.start_table();
    TileBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Tile<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum ViewUpdateOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct ViewUpdate<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ViewUpdate<'a> {
    type Inner = ViewUpdate<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> ViewUpdate<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ViewUpdate {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ViewUpdateArgs<'args>) -> flatbuffers::WIPOffset<ViewUpdate<'bldr>> {
      let mut builder = ViewUpdateBuilder::new(_fbb);
      if let Some(x) = args.tiles { builder.add_tiles(x); }
      if let Some(x) = args.data { builder.add_data(x); }
      builder.add_sqn(args.sqn);
      builder.add_incremental(args.incremental);
      builder.finish()
    }

    pub const VT_SQN: flatbuffers::VOffsetT = 4;
    pub const VT_INCREMENTAL: flatbuffers::VOffsetT = 6;
    pub const VT_DATA: flatbuffers::VOffsetT = 8;
    pub const VT_TILES: flatbuffers::VOffsetT = 10;

  #[inline]
  pub fn sqn(&self) -> u32 {
    self._tab.get::<u32>(ViewUpdate::VT_SQN, Some(0)).unwrap()
  }
  #[inline]
  pub fn incremental(&self) -> bool {
    self._tab.get::<bool>(ViewUpdate::VT_INCREMENTAL, Some(true)).unwrap()
  }
  #[inline]
  pub fn data(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(ViewUpdate::VT_DATA, None).map(|v| v.safe_slice())
  }
  #[inline]
  pub fn tiles(&self) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Tile<'a>>>> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<flatbuffers::ForwardsUOffset<Tile<'a>>>>>(ViewUpdate::VT_TILES, None)
  }
}

pub struct ViewUpdateArgs<'a> {
    pub sqn: u32,
    pub incremental: bool,
    pub data: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a ,  u8>>>,
    pub tiles: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a , flatbuffers::ForwardsUOffset<Tile<'a >>>>>,
}
impl<'a> Default for ViewUpdateArgs<'a> {
    #[inline]
    fn default() -> Self {
        ViewUpdateArgs {
            sqn: 0,
            incremental: true,
            data: None,
            tiles: None,
        }
    }
}
pub struct ViewUpdateBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ViewUpdateBuilder<'a, 'b> {
  #[inline]
  pub fn add_sqn(&mut self, sqn: u32) {
    self.fbb_.push_slot::<u32>(ViewUpdate::VT_SQN, sqn, 0);
  }
  #[inline]
  pub fn add_incremental(&mut self, incremental: bool) {
    self.fbb_.push_slot::<bool>(ViewUpdate::VT_INCREMENTAL, incremental, true);
  }
  #[inline]
  pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ViewUpdate::VT_DATA, data);
  }
  #[inline]
  pub fn add_tiles(&mut self, tiles: flatbuffers::WIPOffset<flatbuffers::Vector<'b , flatbuffers::ForwardsUOffset<Tile<'b >>>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ViewUpdate::VT_TILES, tiles);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ViewUpdateBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ViewUpdateBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ViewUpdate<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum MsgOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct Msg<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Msg<'a> {
    type Inner = Msg<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> Msg<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Msg {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args MsgArgs<'args>) -> flatbuffers::WIPOffset<Msg<'bldr>> {
      let mut builder = MsgBuilder::new(_fbb);
      if let Some(x) = args.content { builder.add_content(x); }
      if let Some(x) = args.session { builder.add_session(x); }
      builder.add_content_type(args.content_type);
      builder.finish()
    }

    pub const VT_SESSION: flatbuffers::VOffsetT = 4;
    pub const VT_CONTENT_TYPE: flatbuffers::VOffsetT = 6;
    pub const VT_CONTENT: flatbuffers::VOffsetT = 8;

  #[inline]
  pub fn session(&self) -> Option<&'a str> {
    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(Msg::VT_SESSION, None)
  }
  #[inline]
  pub fn content_type(&self) -> Content {
    self._tab.get::<Content>(Msg::VT_CONTENT_TYPE, Some(Content::NONE)).unwrap()
  }
  #[inline]
  pub fn content(&self) -> Option<flatbuffers::Table<'a>> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Table<'a>>>(Msg::VT_CONTENT, None)
  }
  #[inline]
  #[allow(non_snake_case)]
  pub fn content_as_view_start(&self) -> Option<ViewStart<'a>> {
    if self.content_type() == Content::ViewStart {
      self.content().map(|u| ViewStart::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn content_as_view_update(&self) -> Option<ViewUpdate<'a>> {
    if self.content_type() == Content::ViewUpdate {
      self.content().map(|u| ViewUpdate::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn content_as_view_ack(&self) -> Option<ViewAck<'a>> {
    if self.content_type() == Content::ViewAck {
      self.content().map(|u| ViewAck::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn content_as_view_end(&self) -> Option<ViewEnd<'a>> {
    if self.content_type() == Content::ViewEnd {
      self.content().map(|u| ViewEnd::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn content_as_broadcast(&self) -> Option<Broadcast<'a>> {
    if self.content_type() == Content::Broadcast {
      self.content().map(|u| Broadcast::init_from_table(u))
    } else {
      None
    }
  }

}

pub struct MsgArgs<'a> {
    pub session: Option<flatbuffers::WIPOffset<&'a  str>>,
    pub content_type: Content,
    pub content: Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>,
}
impl<'a> Default for MsgArgs<'a> {
    #[inline]
    fn default() -> Self {
        MsgArgs {
            session: None,
            content_type: Content::NONE,
            content: None,
        }
    }
}
pub struct MsgBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> MsgBuilder<'a, 'b> {
  #[inline]
  pub fn add_session(&mut self, session: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Msg::VT_SESSION, session);
  }
  #[inline]
  pub fn add_content_type(&mut self, content_type: Content) {
    self.fbb_.push_slot::<Content>(Msg::VT_CONTENT_TYPE, content_type, Content::NONE);
  }
  #[inline]
  pub fn add_content(&mut self, content: flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Msg::VT_CONTENT, content);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> MsgBuilder<'a, 'b> {
    let start = _fbb.start_table();
    MsgBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Msg<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

#[inline]
pub fn get_root_as_msg<'a>(buf: &'a [u8]) -> Msg<'a> {
  flatbuffers::get_root::<Msg<'a>>(buf)
}

#[inline]
pub fn get_size_prefixed_root_as_msg<'a>(buf: &'a [u8]) -> Msg<'a> {
  flatbuffers::get_size_prefixed_root::<Msg<'a>>(buf)
}

#[inline]
pub fn finish_msg_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<Msg<'a>>) {
  fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_msg_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<Msg<'a>>) {
  fbb.finish_size_prefixed(root, None);
}
}  // pub mod switchboard

