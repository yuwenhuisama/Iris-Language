#ifndef _H_IRISVERTUALCODESTRUCTURES_
#define _H_IRISVERTUALCODESTRUCTURES_

typedef char	IR_BYTE;
typedef short	IR_WORD;
typedef int		IR_DWORD;

/*
寻址方式：
立即数寻址（String Integer Float）	imm[]
常量寻址							con[]
全局变量寻址						gv[]
类变量寻址							cv[]
实例变量寻址						iv[]
局部变量寻址						lv[]
成员寻址							mv[]
自成员寻址							sv[]
索引寻址							inv[]
标识符寻址							id[]
寄存器寻址							rv[]
扩展寻址							ex[]
*/

enum class IrisAMType {
	ImmediateString = 0,
	ImmediateInteger,
	ImmediateFloat,
	ImmediateUniqueString,
	Constance,
	GlobalValue,
	ClassValue,
	InstanceValue,
	LocalValue,
	MemberValue,
	//SelfMemberValue,
	IndexValue,
	RegistValue,
	Identifier,
	Extends,
	CastValue,
};

struct IrisVirtualCode {				// Iris虚拟机指令
	IR_BYTE m_bInstructor = 0x00;		// 指令编号
	IR_BYTE m_bOperatorCount = 0x00;	// 操作数个数
	IR_WORD m_wLineNumber = 0x0002;     // 行号

	IR_WORD operator [] (size_t nIndex) {
		if (nIndex < 0 || nIndex > 1) {
			return 0x0000;
		}
		else {
			return nIndex == 0 ? ((0x0000 | m_bInstructor) << 8 | m_bOperatorCount) : m_wLineNumber;
		}
	}

	IrisVirtualCode(IR_BYTE bInstructor, IR_BYTE bOperatorCount) : m_bInstructor(bInstructor), m_bOperatorCount(bOperatorCount) {}
};

struct IrisAM {						// Iris操作数寻址方式
	IR_BYTE m_bType = 0x00;			// 类型
	IR_DWORD m_dwIndex = 0x00;		// 索引

	IR_WORD operator [] (unsigned int nIndex) {
		if (nIndex < 0 || nIndex > 2) {
			return 0x0000;
		}
		else {
			return nIndex == 0 ? m_bType : ((IR_WORD*)(&m_dwIndex))[nIndex - 1];
		}
	}

	IrisAM(IR_BYTE bType, IR_DWORD bIndex) : m_bType(bType), m_dwIndex(bIndex)  {}
	IrisAM() = default;
};

//#define ENDING_AM 

#endif