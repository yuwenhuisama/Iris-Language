#ifndef _H_IRISINSTRUCTUREMAKER_
#define _H_IRISINSTRUCTUREMAKER_

#include "IrisCompiler.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include <list>
using namespace std;

class IrisInstructorMaker
{
private:
	typedef list<IR_DWORD> IR_DWORDS;

public:
	static void push_env();
	static void pop_env();
	static void push();
	static void pop(IR_DWORD bIndex);
	static void cre_env();
	static void load(IrisAMType eType, IR_DWORD dwIndex);
	static void nol_call(IR_DWORD dwIndex, IR_BYTE bParameterCount);
	static void assign(IrisAMType eType, IR_DWORD bIndex);
	static void hid_call(IR_DWORD dwIndex, IR_BYTE bParameterCount);
	static void set_fld(IR_DWORDS& lsFieldMember);
	static void clr_fld();
	static void fld_load(IR_DWORD dwIndex, IR_BYTE bEmptyFlag);
	static void load_nil();
	static void load_true();
	static void load_false();
	static void load_self();
	static void imth_def(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex);
	static void cmth_def(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex);
	static void blk_def(IR_DWORD dwIndex);
	static void end_def(IR_DWORD dwIndex);
	static void jfon(IR_DWORD dwOffset);
	static void jmp(IR_DWORD dwOffset);
	static void ini_tm();
	static void ini_cnt();
	static void cmp_tac();
	static void inc_cnt();
	static void assign_log(IR_DWORD dwParameterCount);
	static void brk();
	static void push_deep(IR_DWORD dwParameterCount);
	static void pop_deep();
	static void rtn();
	static void ctn();
	static void assign_vsl();
	static void assign_iter();
	static void load_iter();
	static void jt(IR_DWORD dwOffset);
	static void assign_cmp();
	static void cmp_cmp();
	static void cre_cenv(IR_BYTE bType);
	static void def_cls(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	static void add_ext();
	static void add_mld();
	static void add_inf();
	static void push_cnt();
	static void push_tim();
	static void pop_cnt(IR_BYTE bDeep);
	static void pop_tim(IR_BYTE bDeep);
	static void push_unim();
	static void pop_unim(IR_BYTE bDeep);
	static void push_vsl();
	static void pop_vsl(IR_BYTE bDeep);
	static void push_iter();
	static void pop_iter(IR_BYTE bDeep);
	static void str_def(IR_DWORD dwNameIndex, IR_DWORD dwParamIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex);
	static void gtr_def(IR_DWORD dwNameIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex);
	static void gstr_def(IR_DWORD dwNameIndex);
	static void set_auth(IR_DWORD dwNameIndex, IR_BYTE bEnv, IR_BYTE bTar, IR_BYTE bType);
	static void def_mld(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	static void def_inf(IR_DWORD dwNameIndex, IR_DWORD dwIndex);
	static void def_infs(IR_DWORD dwNameIndex, IR_DWORDS& lsParameters, IR_DWORD dwVariableParameterIndex);
	static void cblk_def(IR_DWORDS& lsParameters, IR_DWORD dwIndex);
	static void blk();
	static void cast(IR_DWORD nParametersCount);
	static void reg_irp(IR_BYTE bWithIgnoreBlock, IR_DWORD nIndex);
	static void ureg_irp(IR_DWORD nIndex);
	static void assign_ir(IR_DWORD dwIndex);
	static void grn();
	static void spr(IR_BYTE bParameterCount);

	inline static IR_BYTE AMCode(IrisAMType eType) { return (IR_BYTE)eType; }
	inline static void AddInstructorCode(IR_BYTE bInstructor, IR_BYTE bOperatorCount) { IrisCompiler::CurrentCompiler()->AddCode(IrisVirtualCode(bInstructor, bOperatorCount));}
	inline static void AddAMCode(IrisAMType eType, IR_DWORD dwIndex) { IrisCompiler::CurrentCompiler()->AddCode(IrisAM(AMCode(eType), dwIndex)); }

#ifdef IR_DEBUG_PRINT
	static const string GetAMString(IrisAMType eType);
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