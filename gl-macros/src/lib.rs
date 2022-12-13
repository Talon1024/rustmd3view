use proc_macro::TokenStream;

enum DataType {
	Float,
	FloatVec2,
	FloatVec3,
	FloatVec4,
	FloatMatrix2x2,
	FloatMatrix2x3,
	FloatMatrix2x4,
	FloatMatrix3x2,
	FloatMatrix3x3,
	FloatMatrix3x4,
	FloatMatrix4x2,
	FloatMatrix4x3,
	FloatMatrix4x4,
	Int,
	IntVec2,
	IntVec3,
	IntVec4,
	UInt,
	UIntVec2,
	UIntVec3,
	UIntVec4,
	Bool,
	BoolVec2,
	BoolVec3,
	BoolVec4,
}

struct AttributeDefinition {
	name: String,
	size: usize,
	data_type: DataType,
}

struct UniformDefinition {
	mutable: bool,
	name: String,
	data_type: DataType,
}

#[proc_macro]
pub fn model_data(v: TokenStream) -> TokenStream {
	v
}
