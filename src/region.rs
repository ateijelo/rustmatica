use std::{borrow::Cow, ops::RangeInclusive};

use fastnbt::LongArray;

use crate::{
    schema,
    util::{UVec3, Vec3},
    BlockState, Entity, Litematic, TileEntity,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Region<'l> {
    pub name: Cow<'l, str>,
    pub corner1: Vec3,
    pub corner2: Vec3,
    pub tile_entities: Vec<TileEntity<'l>>,
    pub entities: Vec<Entity<'l>>,
    palette: Vec<BlockState<'l>>,
    blocks: Vec<usize>,
}

impl<'l> Region<'l> {
    pub fn new(name: Cow<'l, str>, corner1: Vec3, corner2: Vec3) -> Self {
        let size = (corner1 - corner2).volume();
        Self {
            name,
            corner1,
            corner2,
            tile_entities: vec![],
            entities: vec![],
            palette: vec![block!()],
            blocks: vec![0; size],
        }
    }

    pub fn from_raw(raw: Cow<'l, schema::Region<'_>>, name: Cow<'l, str>) -> Self {
        let corner1 = raw.position;
        let corner2 = raw.position + raw.size - raw.size.signum();

        let mut new = Self::new(name, corner1, corner2);
        new.palette = raw.block_state_palette.to_owned();
        new.tile_entities = raw.tile_entities.to_owned();
        new.entities = raw.entities.to_owned();

        let num_bits = new.num_bits();
        new.blocks = raw
            .block_states
            .iter()
            .flat_map(|block| (0..64).map(move |bit| block >> bit & 1))
            .collect::<Vec<i64>>()
            .chunks(num_bits)
            .map(|slice| {
                slice
                    .iter()
                    .rev()
                    .fold(0, |acc, bit| acc << 1 | *bit as usize)
            })
            .collect::<Vec<usize>>();

        new
    }

    pub fn to_raw(&self) -> schema::Region<'_> {
        let s = self.corner2 - self.corner1;
        let mut new = schema::Region {
            position: self.corner1,
            size: s + s.signum(),
            block_state_palette: self.palette.to_owned(),
            tile_entities: self.tile_entities.to_owned(),
            entities: self.entities.to_owned(),
            pending_fluid_ticks: vec![],
            pending_block_ticks: vec![],
            block_states: LongArray::new(vec![]),
        };

        let num_bits = self.num_bits();
        new.block_states = LongArray::new(
            self.blocks
                .iter()
                .flat_map(|id| (0..num_bits).map(move |bit| id >> bit & 1))
                .collect::<Vec<usize>>()
                .chunks(64)
                .map(|bits| bits.iter().rev().fold(0, |acc, bit| acc << 1 | *bit as i64))
                .collect(),
        );

        new
    }

    fn num_bits(&self) -> usize {
        let mut num_bits = 2;
        while 1 << num_bits < self.palette.len() {
            num_bits += 1;
        }
        num_bits
    }

    fn size(&self) -> UVec3 {
        self.corner1.size_to(&self.corner2)
    }

    fn index_to_pos(&self, index: usize) -> Vec3 {
        let i = index as u32;
        let s = self.size();
        let p = UVec3 {
            x: i % s.x,
            z: (i / s.x) % s.z,
            y: i / (s.z * s.x),
        };
        self.blocks_origin() + p
    }

    fn blocks_origin(&self) -> Vec3 {
        Vec3 {
            x: self.min_x(),
            y: self.min_y(),
            z: self.min_z(),
        }
    }

    fn pos_to_index(&self, pos: Vec3) -> usize {
        let r = (pos - self.blocks_origin()).abs();
        let s = self.size();
        (r.y * s.x * s.z + r.z * s.x + r.x) as usize
    }

    pub fn get_block(&'l self, pos: Vec3) -> &'l BlockState<'_> {
        let blocks_idx = self.pos_to_index(pos);
        &self.palette[self.blocks[blocks_idx]]
    }

    pub fn set_block(&mut self, pos: Vec3, block: BlockState<'l>) {
        let blocks_idx = self.pos_to_index(pos);
        let palette_idx = if let Some(idx) = self.palette.iter().position(|b| b == &block) {
            idx
        } else {
            self.palette.push(block);
            self.palette.len() - 1
        };
        self.blocks[blocks_idx] = palette_idx;
    }

    pub fn get_tile_entity(&'l self, pos: UVec3) -> Option<&'l TileEntity<'_>> {
        self.tile_entities.iter().find(|e| e.pos == pos)
    }

    pub fn set_tile_entity(&mut self, tile_entity: TileEntity<'l>) {
        if let Some(index) = self
            .tile_entities
            .iter()
            .position(|e| e.pos == tile_entity.pos)
        {
            self.tile_entities[index] = tile_entity;
        } else {
            self.tile_entities.push(tile_entity);
        }
    }

    pub fn remove_tile_entity(&mut self, pos: UVec3) {
        if let Some(index) = self.tile_entities.iter().position(|e| e.pos == pos) {
            self.tile_entities.remove(index);
        }
    }

    pub fn min_x(&self) -> i32 {
        self.corner1.x.min(self.corner2.x)
    }

    pub fn max_x(&self) -> i32 {
        self.corner1.x.max(self.corner2.x)
    }

    pub fn min_y(&self) -> i32 {
        self.corner1.y.min(self.corner2.y)
    }

    pub fn max_y(&self) -> i32 {
        self.corner1.y.max(self.corner2.y)
    }

    pub fn min_z(&self) -> i32 {
        self.corner1.z.min(self.corner2.z)
    }

    pub fn max_z(&self) -> i32 {
        self.corner1.z.max(self.corner2.z)
    }

    pub fn x_range(&self) -> RangeInclusive<i32> {
        self.min_x()..=self.max_x()
    }

    pub fn y_range(&self) -> RangeInclusive<i32> {
        self.min_y()..=self.max_y()
    }

    pub fn z_range(&self) -> RangeInclusive<i32> {
        self.min_z()..=self.max_z()
    }

    pub fn volume(&self) -> u32 {
        self.corner1.volume_to(&self.corner2)
    }

    pub fn total_blocks(&self) -> usize {
        self.blocks.iter().filter(|b| b != &&0).count()
    }

    pub fn blocks(&'l self) -> Blocks<'l> {
        Blocks::new(self)
    }

    pub fn as_litematic(self, description: Cow<'l, str>, author: Cow<'l, str>) -> Litematic<'l> {
        let mut l = Litematic::new(self.name.clone(), description, author);
        l.regions.push(self);
        l
    }
}

pub struct Blocks<'b> {
    region: &'b Region<'b>,
    index: usize,
}

impl<'b> Blocks<'b> {
    pub fn new(region: &'b Region<'b>) -> Self {
        Self { region, index: 0 }
    }
}

impl<'b> Iterator for Blocks<'b> {
    type Item = (Vec3, &'b BlockState<'b>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.region.volume() as usize {
            return None;
        }
        let block = self
            .region
            .palette
            .get(*self.region.blocks.get(self.index)?)?;

        let pos = self.region.index_to_pos(self.index);
        self.index += 1;
        Some((pos, block))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::util::Vec3;
    use crate::Region;

    #[test]
    fn test_pos_to_index() {
        let r = Region::new(Cow::from("region"), Vec3::new(0, 0, 0), Vec3::new(2, 2, 2));
        assert_eq!(r.pos_to_index(Vec3::new(0, 0, 0)), 0);
        assert_eq!(r.pos_to_index(Vec3::new(1, 1, 1)), 13);
        assert_eq!(r.pos_to_index(Vec3::new(2, 2, 2)), 26);

        let r = Region::new(Cow::from(""), Vec3::new(-1, -1, -1), Vec3::new(3, 3, 3));
        assert_eq!(r.pos_to_index(Vec3::new(-1, -1, -1)), 0);

        let r = Region::new(Cow::from(""), Vec3::new(1, 1, 1), Vec3::new(-1, -1, -1));
        assert_eq!(r.pos_to_index(Vec3::new(-1, -1, -1)), 0);
        assert_eq!(r.pos_to_index(Vec3::new(0, 0, 0)), 13);
        assert_eq!(r.pos_to_index(Vec3::new(1, 1, 1)), 26);

        let r = Region::new(Cow::from(""), Vec3::new(0, 0, 0), Vec3::new(384, 76, 204));
        assert_eq!(r.pos_to_index(Vec3::new(29, 3, 28)), 247584);
    }

    #[test]
    fn test_index_to_pos() {
        let r = Region::new(Cow::from(""), Vec3::new(0, 0, 0), Vec3::new(2, 2, 2));
        assert_eq!(r.index_to_pos(0), Vec3::new(0, 0, 0));
        assert_eq!(r.index_to_pos(13), Vec3::new(1, 1, 1));
        assert_eq!(r.index_to_pos(26), Vec3::new(2, 2, 2));

        let r = Region::new(Cow::from(""), Vec3::new(-1, -1, -1), Vec3::new(3, 3, 3));
        assert_eq!(r.index_to_pos(0), Vec3::new(-1, -1, -1));

        let r = Region::new(Cow::from(""), Vec3::new(1, 1, 1), Vec3::new(-1, -1, -1));
        assert_eq!(r.index_to_pos(0), Vec3::new(-1, -1, -1));
        assert_eq!(r.index_to_pos(13), Vec3::new(0, 0, 0));
        assert_eq!(r.index_to_pos(26), Vec3::new(1, 1, 1));

        let r = Region::new(Cow::from(""), Vec3::new(0, 0, 0), Vec3::new(384, 76, 204));
        assert_eq!(r.index_to_pos(247584), Vec3::new(29, 3, 28));
    }
}
