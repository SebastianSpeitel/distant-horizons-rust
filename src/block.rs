use std::cell::OnceCell;

use smol_str::SmolStr;

pub mod map_color;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Namespace<'a> {
    #[default]
    Minecraft,
    Other(&'a str),
}

impl AsRef<str> for Namespace<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            Namespace::Minecraft => "minecraft",
            Namespace::Other(ns) => ns,
        }
    }
}

impl<'a> From<&'a str> for Namespace<'a> {
    #[inline]
    fn from(ns: &'a str) -> Self {
        match ns {
            "minecraft" => Namespace::Minecraft,
            ns => Namespace::Other(ns),
        }
    }
}

pub trait Block {
    fn full_identifier(&self) -> &str;

    #[inline]
    fn namespace(&self) -> Namespace<'_> {
        match self.full_identifier().split_once(':') {
            Some(("minecraft", _)) => Namespace::Minecraft,
            Some((ns, _)) => Namespace::Other(ns),
            None => Namespace::Minecraft,
        }
    }

    #[inline]
    fn id(&self) -> &str {
        let id = self.full_identifier();
        match id.split_once(':') {
            Some((_, id)) => id,
            None => id,
        }
    }

    #[inline]
    fn namespace_id(&self) -> (Namespace<'_>, &str) {
        let id = self.full_identifier();
        match id.split_once(':') {
            Some(("minecraft", id)) => (Namespace::Minecraft, id),
            Some((ns, id)) => (Namespace::Other(ns), id),
            None => (Namespace::Minecraft, id),
        }
    }

    #[inline]
    fn is_air(&self) -> bool {
        self.id().eq_ignore_ascii_case("air")
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        default_is_transparent(self)
    }

    #[inline]
    fn get_state(&self, state: &str) -> Option<&str> {
        let _ = state;
        None
    }

    #[inline]
    fn map_color(&self) -> Option<(u8, u8, u8)> {
        map_color::get(self.full_identifier())
    }
}

impl Block for &str {
    #[inline]
    fn full_identifier(&self) -> &str {
        self
    }
}

impl Block for String {
    #[inline]
    fn full_identifier(&self) -> &str {
        self.as_str()
    }
}

impl Block for SmolStr {
    #[inline]
    fn full_identifier(&self) -> &str {
        self.as_str()
    }
}

#[allow(clippy::missing_trait_methods)]
impl<B: Block + ?Sized> Block for Box<B> {
    #[inline]
    fn full_identifier(&self) -> &str {
        self.as_ref().full_identifier()
    }

    #[inline]
    fn id(&self) -> &str {
        self.as_ref().id()
    }

    #[inline]
    fn namespace(&self) -> Namespace<'_> {
        self.as_ref().namespace()
    }

    #[inline]
    fn namespace_id(&self) -> (Namespace<'_>, &str) {
        self.as_ref().namespace_id()
    }

    #[inline]
    fn is_air(&self) -> bool {
        self.as_ref().is_air()
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.as_ref().is_transparent()
    }

    #[inline]
    fn get_state(&self, state: &str) -> Option<&str> {
        self.as_ref().get_state(state)
    }

    #[inline]
    fn map_color(&self) -> Option<(u8, u8, u8)> {
        self.as_ref().map_color()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CachedBlock<B: Block> {
    inner: B,
    namespace: OnceCell<SmolStr>,
    id: OnceCell<SmolStr>,
    is_air: OnceCell<bool>,
    is_transparent: OnceCell<bool>,
    map_color: OnceCell<Option<(u8, u8, u8)>>,
}

impl<B: Block> CachedBlock<B> {
    #[inline]
    pub fn new(block: B) -> Self {
        Self {
            inner: block,
            id: OnceCell::new(),
            namespace: OnceCell::new(),
            is_air: OnceCell::new(),
            is_transparent: OnceCell::new(),
            map_color: OnceCell::new(),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl<B: Block> Block for CachedBlock<B> {
    #[inline]
    fn full_identifier(&self) -> &str {
        &self.inner.full_identifier()
    }

    #[inline]
    fn id(&self) -> &str {
        self.id.get_or_init(|| self.inner.id().into()).as_ref()
    }

    #[inline]
    fn namespace(&self) -> Namespace<'_> {
        self.namespace
            .get_or_init(|| self.inner.namespace().as_ref().into())
            .as_str()
            .into()
    }

    #[inline]
    fn namespace_id(&self) -> (Namespace<'_>, &str) {
        (self.namespace(), self.inner.id())
    }

    #[inline]
    fn is_air(&self) -> bool {
        *self.is_air.get_or_init(|| self.inner.is_air())
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        *self
            .is_transparent
            .get_or_init(|| self.inner.is_transparent())
    }

    #[inline]
    fn get_state(&self, state: &str) -> Option<&str> {
        self.inner.get_state(state)
    }

    #[inline]
    fn map_color(&self) -> Option<(u8, u8, u8)> {
        *self.map_color.get_or_init(|| self.inner.map_color())
    }
}

#[inline]
fn default_is_transparent(block: &(impl Block + ?Sized)) -> bool {
    use Namespace::*;
    match block.namespace_id() {
        (Minecraft, "AIR") => true,
        (Minecraft, "torch") => true,
        (Minecraft, "wall_torch") => true,
        (Minecraft, "rail") => true,
        (Minecraft, "powered_rail") => true,
        (Minecraft, "lever") => true,
        (Minecraft, "ladder") => true,
        (Minecraft, "glass") => true,
        (Minecraft, "repeater") => true,
        (Minecraft, "iron_bars") => true,
        (Minecraft, "redstone_wire") => true,
        (Minecraft, "end_rod") => true,
        (Minecraft, b) if b.starts_with("potted_") => true,
        _ => false,
    }
}

impl core::fmt::Debug for dyn Block + Send + Sync {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("namespace", &self.namespace())
            .field("id", &self.id())
            .finish()
    }
}
