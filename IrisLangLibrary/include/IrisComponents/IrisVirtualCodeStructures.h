#ifndef _H_IRISVERTUALCODESTRUCTURES_
#define _H_IRISVERTUALCODESTRUCTURES_

typedef char	IR_BYTE;
typedef short	IR_WORD;
typedef int		IR_DWORD;

/*
Ѱַ��ʽ��
������Ѱַ��String Integer Float��	imm[]
����Ѱַ							con[]
ȫ�ֱ���Ѱַ						gv[]
�����Ѱַ							cv[]
ʵ������Ѱַ						iv[]
�ֲ�����Ѱַ						lv[]
��ԱѰַ							mv[]
�Գ�ԱѰַ							sv[]
����Ѱַ							inv[]
��ʶ��Ѱַ							id[]
�Ĵ���Ѱַ							rv[]
��չѰַ							ex[]
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

struct IrisVirtualCode {				// Iris�����ָ��
	IR_BYTE m_bInstructor = 0x00;		// ָ����
	IR_BYTE m_bOperatorCount = 0x00;	// ����������
	IR_WORD m_wLineNumber = 0x0002;     // �к�

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

struct IrisAM {						// Iris������Ѱַ��ʽ
	IR_BYTE m_bType = 0x00;			// ����
	IR_DWORD m_dwIndex = 0x00;		// ����

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