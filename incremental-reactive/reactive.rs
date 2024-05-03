/// https://en.wikipedia.org/wiki/Plane_(Dungeons_%26_Dragons)
#[derive(Default)]
pub struct PLANE {
    storages: FastHashMap<TypeId, Box<dyn ShrinkableAny>>,
}

impl<T: IncrementalBase> ShrinkableAny for IncrementalSignalStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn shrink_to_fit(&mut self) {
        self.inner.data.write().shrink_to_fit();
        self.inner.sub_watchers.write().shrink_to_fit();
    }
}

static ACTIVE_PLANE: parking_lot::RwLock<Option<PLANE>> = parking_lot::RwLock::new(None);
pub fn setup_active_plane(sg: PLANE) -> Option<PLANE> {
    ACTIVE_PLANE.write().replace(sg)
}
pub fn shrink_active_plane() {
    let mut plane = ACTIVE_PLANE.write();
    if let Some(plane) = plane.as_mut() {
        plane.storages.values_mut().for_each(|s| s.shrink_to_fit())
    }
}

pub fn access_storage_of<T: IncrementalBase, R>(
    acc: impl FnOnce(&IncrementalSignalStorage<T>) -> R,
) -> R {
    let id = TypeId::of::<T>();

    // not add write lock first if the storage exists
    let try_read_storages = ACTIVE_PLANE.read();
    let storages = try_read_storages
        .as_ref()
        .expect("global storage group not specified");
    if let Some(storage) = storages.storages.get(&id) {
        let storage = storage
            .as_ref()
            .as_any()
            .downcast_ref::<IncrementalSignalStorage<T>>()
            .unwrap();
        acc(storage)
    } else {
        drop(try_read_storages);
        let mut storages = ACTIVE_PLANE.write();
        let storages = storages
            .as_mut()
            .expect("global storage group not specified");
        let storage = storages
            .storages
            .entry(id)
            .or_insert_with(|| Box::<IncrementalSignalStorage<T>>::default());
        let storage = storage
            .as_ref()
            .as_any()
            .downcast_ref::<IncrementalSignalStorage<T>>()
            .unwrap();
        acc(storage)
    }
}
pub fn storage_of<T: IncrementalBase>() -> IncrementalSignalStorage<T> {
    access_storage_of(|s| s.clone())
}



trait ModifyIdentityDelta<T: ApplicableIncremental> {
  fn apply(self, target: &mut IncrementalSignal<T>);
}

impl<T, X> ModifyIdentityDelta<T> for X
where
  T: ApplicableIncremental<Delta = X>,
{
  fn apply(self, target: &mut IncrementalSignal<T>) {
    target.mutate(|mut m| {
      m.modify(self);
    })
  }
}

/// An wrapper struct that prevent outside directly accessing the mutable T, but have to modify
it /// through the explicit delta type. When modifying, the delta maybe checked if is really
valid by /// diffing, and the change will be collect by a internal collector
pub struct Mutating<'a, T: IncrementalBase> {
  inner: &'a mut T,
  collector: &'a mut dyn FnMut(&T::Delta, &T),
}

impl<'a, T: IncrementalBase> Mutating<'a, T> {
  pub fn new(inner: &'a mut T, collector: &'a mut dyn FnMut(&T::Delta, &T)) -> Self {
    Self { inner, collector }
  }
}

impl<'a, T: IncrementalBase> Deref for Mutating<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.inner
  }
}

impl<'a, T: ApplicableIncremental> Mutating<'a, T> {
  pub fn modify(&mut self, delta: T::Delta) {
    if self.inner.should_apply_hint(&delta) {
      (self.collector)(&delta, self.inner);
      self.inner.apply(delta).unwrap()
    }
  }
}

impl<'a, T: IncrementalBase> Mutating<'a, T> {
  /// # Safety
  /// the mutation should be record manually, and will not triggered in the collector
  pub unsafe fn get_mut_ref(&mut self) -> &mut T {
    self.inner
  }

  /// # Safety
  /// the mutation will be not apply on original data but only triggered in the collector
  pub unsafe fn trigger_change_but_not_apply(&mut self, delta: T::Delta) {
    (self.collector)(&delta, self.inner);
  }
}

pub trait GlobalIdReactiveMapping<M> {
  type ChangeStream: Stream + Unpin + 'static;
  type Ctx<'a>;

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream);

  fn update(&self, mapped: &mut M, change: &mut Self::ChangeStream, ctx: &Self::Ctx<'_>);
}

pub trait GlobalIdReactiveSimpleMapping<M> {
  type ChangeStream: Stream + Unpin + 'static;
  type Ctx<'a>;

  fn build(&self, ctx: &Self::Ctx<'_>) -> (M, Self::ChangeStream);
}
