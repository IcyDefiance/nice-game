extern crate atom;
extern crate futures;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;

pub mod cpu_pool;
pub mod mesh;
pub mod sprite;
pub mod texture;
pub mod window;

pub use vulkano::instance::Version;

use cpu_pool::CpuPool;
use std::{ cmp::min, sync::{ Arc, Mutex, Weak } };
use vulkano::{
	command_buffer::AutoCommandBuffer,
	device::Queue,
	format::Format,
	image::ImageViewAccess,
	instance::{ ApplicationInfo, Instance, InstanceCreationError },
	memory::DeviceMemoryAllocError,
	sync::GpuFuture,
};

lazy_static! {
	static ref CPU_POOL: Mutex<CpuPool> = Mutex::new(CpuPool::new(min(1, num_cpus::get() - 1)));
	static ref FS_POOL: Mutex<CpuPool> = Mutex::new(CpuPool::new(1));
}

pub fn cpu_pool() -> &'static Mutex<CpuPool> {
	&CPU_POOL
}

pub fn fs_pool() -> &'static Mutex<CpuPool> {
	&FS_POOL
}

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
}
impl Context {
	pub fn new(name: Option<&str>, version: Option<Version>) -> Result<Self, InstanceCreationError> {
		Ok(Self {
			instance:
				Instance::new(
					Some(&ApplicationInfo {
						application_name: name.map(|x| x.into()),
						application_version: version,
						engine_name: Some("nIce Game".into()),
						engine_version: Some(Version {
							major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
							minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
							patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
						}),
					}),
					&vulkano_win::required_extensions(),
					None
				)?
		})
	}
}

pub struct ObjectId {
	val: Weak<()>,
}
impl ObjectId {
	pub fn is_child_of(&self, root: &ObjectIdRoot) -> bool {
		self.val.upgrade().map_or(false, |val| Arc::ptr_eq(&val, &root.val))
	}
}

pub struct ObjectIdRoot {
	val: Arc<()>,
}
impl ObjectIdRoot {
	fn new() -> Self {
		Self { val: Arc::default() }
	}

	pub fn make_id(&self) -> ObjectId {
		ObjectId { val: Arc::downgrade(&self.val) }
	}
}

pub trait RenderTarget {
	fn format(&self) -> Format;
	fn id_root(&self) -> &ObjectIdRoot;
	fn images(&self) -> &[Arc<ImageViewAccess + Send + Sync + 'static>];
	fn join_future(&mut self, other: Box<GpuFuture>);
	fn take_future(&mut self) -> Option<Box<GpuFuture>>;
	fn queue(&self) -> &Arc<Queue>;
}

pub trait Drawable {
	fn commands(
		&mut self,
		target: &mut RenderTarget,
		image_num: usize,
	) -> Result<AutoCommandBuffer, DeviceMemoryAllocError>;
}
