use crate::render::IsfInputsUniform;
use bevy::prelude::*;
use bevy::reflect::utility::NonGenericTypeInfoCell;
use bevy::reflect::{
    ApplyError, DynamicStruct, FieldIter, FromType, GetTypeRegistration, NamedField,
    ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned, ReflectRef, StructInfo, TypeInfo,
    TypeRegistration, Typed,
};
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::ops::Deref;

#[derive(Resource, ExtractResource, TypePath, Deref, DerefMut, Debug, Clone, Default)]
pub struct IsfInputs(BTreeMap<String, IsfInputValue>);

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuBool {
    value: u32,
    _padding: [u32; 3], // Padding to make up 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuFloat {
    value: f32,
    _padding: [f32; 3], // Padding to make up 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuLong {
    value: i32,
    _padding: [i32; 3], // Padding to make up 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuPoint2d {
    x: f32,
    y: f32,
    _padding: [f32; 2], // Padding to make up 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32, // No padding needed, already 16 bytes
}

impl IsfInputs {
    pub(crate) fn to_uniform(&self) -> IsfInputsUniform {
        let uniform = IsfInputsUniform(self.serialize_values());
        uniform
    }

    pub(crate) fn uniform_size(&self) -> usize {
        let size = self
            .iter()
            .map(|(_, v)| match v {
                IsfInputValue::Bool(_) => std::mem::size_of::<GpuBool>(),
                IsfInputValue::Float(_) => std::mem::size_of::<GpuFloat>(),
                IsfInputValue::Long(_) => std::mem::size_of::<GpuLong>(),
                IsfInputValue::Point2d(_) => std::mem::size_of::<GpuPoint2d>(),
                IsfInputValue::Color(_) => std::mem::size_of::<GpuColor>(),
                _ => 0,
            })
            .sum();
        if size < 40 {
            40
        } else {
            size
        }
    }

    fn serialize_values(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for (_, value) in self.deref() {
            match value {
                IsfInputValue::Bool(b) => {
                    let gpu_value = GpuBool {
                        value: *b as u32,
                        _padding: [0; 3],
                    };
                    buffer.extend_from_slice(bytemuck::bytes_of(&gpu_value));
                }
                IsfInputValue::Float(f) => {
                    let gpu_value = GpuFloat {
                        value: *f,
                        _padding: [0.0; 3],
                    };
                    buffer.extend_from_slice(bytemuck::bytes_of(&gpu_value));
                }
                IsfInputValue::Long(l) => {
                    let gpu_value = GpuLong {
                        value: *l,
                        _padding: [0; 3],
                    };
                    buffer.extend_from_slice(bytemuck::bytes_of(&gpu_value));
                }
                IsfInputValue::Point2d(v) => {
                    let gpu_value = GpuPoint2d {
                        x: v.x,
                        y: v.y,
                        _padding: [0.0; 2],
                    };
                    buffer.extend_from_slice(bytemuck::bytes_of(&gpu_value));
                }
                IsfInputValue::Color(c) => {
                    let [r, g, b, a] = c.to_linear().to_f32_array();
                    let gpu_value = GpuColor { r, g, b, a };
                    buffer.extend_from_slice(bytemuck::bytes_of(&gpu_value));
                }
                _ => {}
            }
        }

        if buffer.len() < 40 {
            buffer.resize(40, 0);
        }

        buffer
    }
}

#[derive(Debug, Reflect, Clone)]
pub enum IsfInputValue {
    Event(bool),
    Bool(bool),
    Long(i32),
    Float(f32),
    Point2d(Vec2),
    Color(Color),
    Image(Handle<Image>),
    Audio(Vec<f32>),
    AudioFft(Vec<f32>),
}

impl IsfInputs {
    pub fn from_isf(isf: &isf::Isf) -> Self {
        let mut values = BTreeMap::new();
        for input in &isf.inputs {
            let value = match &input.ty {
                isf::InputType::Event => IsfInputValue::Event(false),
                isf::InputType::Bool(b) => IsfInputValue::Bool(b.default.unwrap_or_default()),
                isf::InputType::Long(l) => IsfInputValue::Long(l.default.unwrap_or_default()),
                isf::InputType::Float(f) => IsfInputValue::Float(f.default.unwrap_or_default()),
                isf::InputType::Point2d(p) => {
                    let [x, y] = p.default.unwrap_or_default();
                    IsfInputValue::Point2d(Vec2::new(x, y))
                }
                isf::InputType::Color(c) => {
                    let rgba = c
                        .default
                        .clone()
                        .unwrap_or_else(|| vec![0.0, 0.0, 0.0, 1.0]);
                    IsfInputValue::Color(Color::srgba(
                        rgba[0],
                        rgba[1],
                        rgba[2],
                        *rgba.get(4).unwrap_or(&1.0f32),
                    ))
                }
                isf::InputType::Image => IsfInputValue::Image(Handle::default()),
                isf::InputType::Audio(_) => IsfInputValue::Audio(Vec::new()),
                isf::InputType::AudioFft(_) => IsfInputValue::AudioFft(Vec::new()),
            };
            values.insert(input.name.clone(), value);
        }
        Self(values)
    }
}

impl Typed for IsfInputs {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| {
            let fields = [NamedField::new::<BTreeMap<String, IsfInputValue>>("values")];
            let info = StructInfo::new::<Self>(&fields);
            TypeInfo::Struct(info)
        })
    }
}

impl GetTypeRegistration for IsfInputs {
    fn get_type_registration() -> TypeRegistration {
        let mut type_registration = TypeRegistration::of::<IsfInputs>();
        type_registration.insert::<ReflectFromPtr>(FromType::<IsfInputs>::from_type());
        // type_registration.insert::<ReflectInspectorOptions>(FromType::<IsfInputs>::from_type());
        type_registration
    }
}

impl Struct for IsfInputs {
    fn field(&self, name: &str) -> Option<&dyn PartialReflect> {
        self.get(name).map(|v| match v {
            IsfInputValue::Event(b) => b as &dyn PartialReflect,
            IsfInputValue::Bool(b) => b as &dyn PartialReflect,
            IsfInputValue::Long(l) => l as &dyn PartialReflect,
            IsfInputValue::Float(f) => f as &dyn PartialReflect,
            IsfInputValue::Point2d(v) => v as &dyn PartialReflect,
            IsfInputValue::Color(c) => c as &dyn PartialReflect,
            IsfInputValue::Image(h) => h as &dyn PartialReflect,
            IsfInputValue::Audio(a) => a as &dyn PartialReflect,
            IsfInputValue::AudioFft(a) => a as &dyn PartialReflect,
        })
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect> {
        self.get_mut(name).map(|v| match v {
            IsfInputValue::Event(b) => b as &mut dyn PartialReflect,
            IsfInputValue::Bool(b) => b as &mut dyn PartialReflect,
            IsfInputValue::Long(l) => l as &mut dyn PartialReflect,
            IsfInputValue::Float(f) => f as &mut dyn PartialReflect,
            IsfInputValue::Point2d(v) => v as &mut dyn PartialReflect,
            IsfInputValue::Color(c) => c as &mut dyn PartialReflect,
            IsfInputValue::Image(h) => h as &mut dyn PartialReflect,
            IsfInputValue::Audio(a) => a as &mut dyn PartialReflect,
            IsfInputValue::AudioFft(a) => a as &mut dyn PartialReflect,
        })
    }

    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.values().nth(index).map(|v| match v {
            IsfInputValue::Event(b) => b as &dyn PartialReflect,
            IsfInputValue::Bool(b) => b as &dyn PartialReflect,
            IsfInputValue::Long(l) => l as &dyn PartialReflect,
            IsfInputValue::Float(f) => f as &dyn PartialReflect,
            IsfInputValue::Point2d(v) => v as &dyn PartialReflect,
            IsfInputValue::Color(c) => c as &dyn PartialReflect,
            IsfInputValue::Image(h) => h as &dyn PartialReflect,
            IsfInputValue::Audio(a) => a as &dyn PartialReflect,
            IsfInputValue::AudioFft(a) => a as &dyn PartialReflect,
        })
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.values_mut().nth(index).map(|v| match v {
            IsfInputValue::Event(b) => b as &mut dyn PartialReflect,
            IsfInputValue::Bool(b) => b as &mut dyn PartialReflect,
            IsfInputValue::Long(l) => l as &mut dyn PartialReflect,
            IsfInputValue::Float(f) => f as &mut dyn PartialReflect,
            IsfInputValue::Point2d(v) => v as &mut dyn PartialReflect,
            IsfInputValue::Color(c) => c as &mut dyn PartialReflect,
            IsfInputValue::Image(h) => h as &mut dyn PartialReflect,
            IsfInputValue::Audio(a) => a as &mut dyn PartialReflect,
            IsfInputValue::AudioFft(a) => a as &mut dyn PartialReflect,
        })
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        self.keys().nth(index).map(|s| s.as_str())
    }

    fn field_len(&self) -> usize {
        self.len()
    }

    fn iter_fields(&self) -> FieldIter {
        FieldIter::new(self)
    }

    fn clone_dynamic(&self) -> DynamicStruct {
        let mut dynamic_struct = DynamicStruct::default();
        for (name, value) in self.deref() {
            dynamic_struct.insert(name, value.clone());
        }
        dynamic_struct
    }
}

impl PartialReflect for IsfInputs {
    #[inline]
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(Self::type_info())
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        if let ReflectRef::Struct(struct_value) = value.reflect_ref() {
            for (i, value) in struct_value.iter_fields().enumerate() {
                let name = struct_value.name_at(i).unwrap();
                if let Some(v) = self.field_mut(name) {
                    v.try_apply(value)?;
                }
            }
        } else {
            return Err(ApplyError::MismatchedKinds {
                from_kind: value.reflect_kind(),
                to_kind: ReflectKind::Struct,
            });
        }
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Struct
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Struct(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Struct(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Struct(self)
    }

    #[inline]
    fn clone_value(&self) -> Box<dyn PartialReflect> {
        Box::new(self.clone_dynamic())
    }

    fn reflect_partial_eq(&self, value: &dyn PartialReflect) -> Option<bool> {
        struct_partial_eq(self, value)
    }

    fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, " IsfInputs(")?;
        for (i, (name, value)) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {:?}", name, value)?;
        }
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Ok(self)
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        Some(self)
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        Some(self)
    }
}

impl Reflect for IsfInputs {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }
}

#[inline]
pub fn struct_partial_eq(a: &IsfInputs, b: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::Struct(struct_value) = b.reflect_ref() else {
        return Some(false);
    };

    if a.len() != struct_value.field_len() {
        return Some(false);
    }

    for (i, value) in struct_value.iter_fields().enumerate() {
        let name = struct_value.name_at(i).unwrap();
        if let Some(field_value) = a.get(name) {
            let eq_result = field_value.reflect_partial_eq(value);
            if let failed @ (Some(false) | None) = eq_result {
                return failed;
            }
        } else {
            return Some(false);
        }
    }

    Some(true)
}
