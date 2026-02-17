// Intended for use with <https://github.com/WerWolv/ImHex>.

#pragma endian little

import type.leb128;
import std.array;

#pragma eval_depth 128

using varuint = type::uLEB128;
using f32 = float;
using f64 = double;

using i16 = s16;
using i32 = s32;

using Ref = varuint;

enum TypeId: u8 {
    String = 0,
    Attributes = 1,
    Axes = 2,
    Bool = 3,
    BrickColor = 4,
    CFrame = 5,
    Color3 = 6,
    Color3uint8 = 7,
    ColorSequence = 8,
    Enum = 9,
    Faces = 10,
    Float32 = 11,
    Float64 = 12,
    Int32 = 13,
    MaterialColors = 14,
    NumberRange = 15,
    NumberSequence = 16,
    None = 17,
    DefaultPhysicalProperties = 18,
    CustomPhysicalProperties = 19,
    Ray = 20,
    Rect = 21,
    Ref = 22,
    Region3 = 23,
    Region3int16 = 24,
    SecurityCapabilities = 25,
    BinaryString = 26,
    Tags = 27,
    UDim = 28,
    UDim2 = 29,
    Vector2 = 30,
    Vector2int16 = 31,
    Vector3 = 32,
    Vector3int16 = 33,
    Font = 34,

    ContentNone = 35,
    ContentObject = 36,
    ContentUri = 37,
};

// variable length array used everywhere
struct VArray<T> {
    varuint size;
    T data[size];
};

using Varstring;
using Nullstring;

fn fmtVarstring(Varstring data) {
    return std::format("\"{}\"", data.data);
};

fn fmtNullstring(Nullstring data) {
    return std::format("\"{}\"", data.data);
};

// like a Rust string
using Varstring = VArray<char> [[sealed, format_read("fmtVarstring"), transform("fmtVarstring")]];

// like a C string
struct Nullstring {
    char data[while(std::mem::read_unsigned($, 1) != 0x00)];
    u8 null_byte [[hidden]];
} [[sealed, format_read("fmtNullstring"), transform("fmtNullstring")]];

using Variant;

struct KeyAndValue {
    Nullstring name;
    Variant data;
};

using Attributes = VArray<KeyAndValue>;

bitfield Axes {
    bool X: 1;
    bool Y: 1;
    bool Z: 1;
};

struct CFrame {
    u8 id;

    if (id == 0) {
        f32 xx;
        f32 xy;
        f32 xz;

        f32 yx;
        f32 yy;
        f32 yz;

        f32 zx;
        f32 zy;
        f32 zz;
        
        f32 positionX, positionY, positionZ;
    } else {
        f32 positionX, positionY, positionZ;
    }
};

struct Color3Template<T> {
    T r, g, b;
};

using Color3 = Color3Template<f32>;
using Color3uint8 = Color3Template<u8>;

struct ColorSequenceKeypoint {
    f32 time, r, g, b;
};

using ColorSequence = VArray<ColorSequenceKeypoint>;

bitfield Faces {
    bool right: 1;
    bool top: 1;
    bool back: 1;
    bool left: 1;
    bool bottom: 1;
    bool front: 1;
};

struct MaterialColors {
    padding[69];
};

struct NumberRange {
    f32 min, max;
};

struct NumberSequenceKeypoint {
    f32 envelope, time, value;
};

using NumberSequence = VArray<NumberSequenceKeypoint>;

struct CustomPhysicalProperties {
    f32 density, elasticity, elasticityWeight, friction, frictionWeight;
};

struct Ray {
    f32 directionX, directionY, directionZ, originX, originY, originZ;
};

struct Rect {
    f32 minX, minY, maxX, maxY;
};

struct RegionTemplate<T> {
    T minX, minY, minZ, maxX, maxY, maxZ;
};

using Region3 = RegionTemplate<f32>;
using Region3int16 = RegionTemplate<i16>;

struct SecurityCapabilities {
    padding[8];
};

using Tags = VArray<Nullstring>;

struct UDim {
    i32 offset;
    f32 scale;
};

struct UDim2 {
    UDim x, y;
};

struct Vector2Template<T> {
    T x, y;
};

using Vector2 = Vector2Template<f32>;
using Vector2int16 = Vector2Template<i16>;


struct Vector3Template<T> {
    T x, y, z;
};

using Vector3 = Vector3Template<f32>;
using Vector3int16 = Vector3Template<i16>;

struct Font {
    Nullstring family;
    u16 weight;
    u8 style;
};

struct Variant {
    TypeId typeId;

    match (typeId) {
        (TypeId::String): Varstring data;
        (TypeId::Attributes): Attributes data;
        (TypeId::Axes): Axes data;
        (TypeId::Bool): bool data;
        (TypeId::BrickColor): Nullstring data;
        (TypeId::CFrame): CFrame data;
        (TypeId::Color3): Color3 data;
        (TypeId::Color3uint8): Color3uint8 data;
        (TypeId::ColorSequence): ColorSequence data;
        (TypeId::Enum): varuint data;
        (TypeId::Faces): Faces data;
        (TypeId::Float32): f32 data;
        (TypeId::Float64): f64 data;
        (TypeId::Int32): i32 data;
        (TypeId::MaterialColors): MaterialColors data;
        (TypeId::NumberRange): NumberRange data;
        (TypeId::NumberSequence): NumberSequence data;
        (TypeId::None): padding[0];
        (TypeId::DefaultPhysicalProperties): padding[0];
        (TypeId::CustomPhysicalProperties): CustomPhysicalProperties data;
        (TypeId::Ray): Ray data;
        (TypeId::Rect): Rect data;
        (TypeId::Ref): Ref data;
        (TypeId::Region3): Region3 data;
        (TypeId::Region3int16): Region3int16 data;
        (TypeId::SecurityCapabilities): SecurityCapabilities data;
        (TypeId::BinaryString): Varstring data;
        (TypeId::Tags): Tags data;
        (TypeId::UDim): UDim data;
        (TypeId::UDim2): UDim2 data;
        (TypeId::Vector2): Vector2 data;
        (TypeId::Vector2int16): Vector2int16 data;
        (TypeId::Vector3): Vector3 data;
        (TypeId::Vector3int16): Vector3int16 data;
        (TypeId::Font): Font data;

        (TypeId::ContentNone): padding[0];
        (TypeId::ContentObject): Ref data;
        (TypeId::ContentUri): Varstring data;
  }
};

struct Properties {
    u16 size;
    KeyAndValue data[size] [[inline]];
};

struct Instance {
    Varstring Name;
    Nullstring ClassName;
    Variant Self;
    Variant Parent;
    Properties Properties;
};

Instance instances[while(!std::mem::eof())] @ 0x00;
