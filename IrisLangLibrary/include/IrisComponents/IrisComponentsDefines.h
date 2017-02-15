#ifndef _H_IRISCOMPONENTSDEFINES_
#define _H_IRISCOMPONENTSDEFINES_

enum class IrisBinaryExpressionType {
	Assign = 0,
	AssignAdd,
	AssignSub,
	AssignMul,
	AssignDiv,
	AssignMod,
	AssignPower,
	AssignBitAnd,
	AssignBitOr,
	AssignBitXor,
	AssignBitShr,
	AssignBitShl,
	AssignBitSar,
	AssignBitSal,

	LogicOr,
	LogicAnd,

	LogicBitOr,
	LogicBitXor,
	LogicBitAnd,

	Equal,
	NotEqual,

	GreatThan,
	GreatThanOrEqual,
	LessThan,
	LessThanOrEqual,

	BitShr,
	BitShl,
	BitSar,
	BitSal,

	Add,
	Sub,
	Mul,
	Div,
	Mod,

	Power,
};

enum IrisUnaryExpressionType {
	LogicNot = 0,
	BitNot,
	Minus,
	Plus,
};

enum class IrisIdentifierType {
	Constance = 0,
	LocalVariable,
	GlobalVariable,
	InstanceVariable,
	ClassVariable,
};

enum class IrisAuthorityEnvironment {
	Class = 0,
	Module,
};

enum class IrisAuthorityTarget {
	InstanceMethod = 0,
	ClassMethod,
};

enum class IrisAuthorityType {
	EveryOne = 0,
	Relative,
	Personal,
};

enum class IrisNativeObjectType {
	String = 0,
	Integer,
	Float,
	UniqueString,
};

enum class IrisInstantValueType {
	Nil = 0,
	True,
	False,
};

enum class IrisRangeType {
	LeftOpenAndRightOpen = 0,
	LeftOpenAndRightClosed,
	LeftClosedAndRightClosed,
	LeftClosedAndRightOpen,
};

#endif