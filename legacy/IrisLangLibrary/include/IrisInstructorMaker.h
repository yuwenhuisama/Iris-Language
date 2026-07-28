#ifndef _H_IRISINSTRUCTUREMAKER_
#define _H_IRISINSTRUCTUREMAKER_

#include "IrisCompiler.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include <list>
#include <functional>

using namespace std;

class IrisInstructorMaker
{
private:
	typedef list<IR_DWORD> IR_DWORDS;
	typedef list<function<void(void)>> _GenerateInstructionList;

private:
	_GenerateInstructionList m_lsGenerateInstructionList;

private : 
	void AddGenerateInstruction(function<void(void)> fInsturction) {
		m_lsGenerateInstructionList.push_back(fInsturction);
	}

	size_t CalcInstructorLength(size_t nParameters) {
		return 2 + nParameters * 3;
	}

public :
	class Label {
	private:
		size_t m_nStartOffset = 0;

	public:

		void SetStartOffset(size_t nOffset);
		size_t GetStartOffset();

		Label() = default;
		~Label() = default;
	};

public:

	void Generate();

	void place_lable(Label* pLabel);

	void push_env();
	void pop_env();
	void push();
	void pop(IR_DWORD bIndex);
	void cre_env();
	void load(IrisAMType eType, IR_DWORD dwIndex);
	void nol_call(IR_DWORD dwIndex, IR_BYTE bParameterCount);
	void assign(IrisAMType eType, IR_DWORD bIndex);
	void hid_call(IR_DWORD dwIndex, IR_BYTE bParameterCount);
	void set_fld(IR_DWORDS& lsFieldMember);
	void clr_fld();
	void fld_load(IR_DWORD dwIndex, IR_BYTE bEmptyFlag);
	void load_nil();
	void load_true();
	void load_false();
	void load_self();
	void imth_def(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex);
	void cmth_def(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex);
	void blk_def(IR_DWORD dwIndex);
	void end_def(IR_DWORD dwIndex);
	void jfon(Label* pLabel);
	void jmp(Label* pLabel);
	void ini_tm();
	void ini_cnt();
	void cmp_tac();
	void inc_cnt();
	void assign_log(IR_DWORD dwParameterCount);
	void brk(Label* pLabel);
	void push_deep(IR_DWORD dwParameterCount);
	void pop_deep();
	void rtn();
	void ctn(Label* pLabel);
	void assign_vsl();
	void assign_iter();
	void load_iter();
	void jt(Label* pLabel);
	void assign_cmp();
	void cmp_cmp();
	void cre_cenv(IR_BYTE bType);
	void def_cls(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	void add_ext();
	void add_mld();
	void add_inf();
	void push_cnt();
	void push_tim();
	void pop_cnt(IR_BYTE bDeep);
	void pop_tim(IR_BYTE bDeep);
	void push_unim();
	void pop_unim(IR_BYTE bDeep);
	void push_vsl();
	void pop_vsl(IR_BYTE bDeep);
	void push_iter();
	void pop_iter(IR_BYTE bDeep);
	void str_def(IR_DWORD dwNameIndex, IR_DWORD dwParamIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex);
	void gtr_def(IR_DWORD dwNameIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex);
	void gstr_def(IR_DWORD dwNameIndex);
	void set_auth(IR_DWORD dwNameIndex, IR_BYTE bEnv, IR_BYTE bTar, IR_BYTE bType);
	void def_mld(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	void def_inf(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	void def_infs(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex);
	void cblk_def(IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex, IR_DWORD dwIndex);
	void blk();
	//void cast(IR_DWORD nParametersCount);
	void reg_irp(IR_BYTE bWithIgnoreBlock, IR_DWORD nIndex);
	void ureg_irp(IR_DWORD nIndex);
	void assign_ir(IR_DWORD dwIndex);
	void grn();
	void spr(IR_BYTE bParameterCount);
	void load_cast();

	inline IR_BYTE AMCode(IrisAMType eType) { return (IR_BYTE)eType; }
	inline void AddInstructorCode(IR_BYTE bInstructor, IR_BYTE bOperatorCount) { IrisCompiler::CurrentCompiler()->AddCode(IrisVirtualCode(bInstructor, bOperatorCount));}
	inline void AddAMCode(IrisAMType eType, IR_DWORD dwIndex) { IrisCompiler::CurrentCompiler()->AddCode(IrisAM(AMCode(eType), dwIndex)); }

#if IR_DEBUG_PRINT
	const string GetAMString(IrisAMType eType);
#endif

private:
	IrisInstructorMaker();
	static IrisInstructorMaker* s_pInstance;

public:	
	static IrisInstructorMaker* CurrentInstructor();

public:
	virtual ~IrisInstructorMaker();
};

#endif