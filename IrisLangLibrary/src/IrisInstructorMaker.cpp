#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisVirtualCodeNumber.h"

#if IR_DEBUG_PRINT
#include <iostream>
#include <iomanip>
using namespace std;
#endif

IrisInstructorMaker* IrisInstructorMaker::s_pInstance = nullptr;

#if IR_DEBUG_PRINT
const string IrisInstructorMaker::GetAMString(IrisAMType eType)
{
	switch (eType)
	{
	case IrisAMType::ImmediateString:
		return "imm_str";
		break;
	case IrisAMType::ImmediateInteger:
		return "imm_int";
		break;
	case IrisAMType::ImmediateFloat:
		return "imm_flt";
		break;
	case IrisAMType::Constance:
		return "con";
		break;
	case IrisAMType::GlobalValue:
		return "gv";
		break;
	case IrisAMType::ClassValue:
		return "cv";
		break;
	case IrisAMType::InstanceValue:
		return "iv";
		break;
	case IrisAMType::LocalValue:
		return "lv";
		break;
	case IrisAMType::MemberValue:
		return "mv";
		break;
	//case IrisAMType::SelfMemberValue:
	//	return "sv";
	//	break;
	case IrisAMType::IndexValue:
		return "inv";
		break;
	case IrisAMType::RegistValue:
		return "rv";
		break;
	case IrisAMType::Identifier:
		return "id";
		break;
	case IrisAMType::Extends:
		return "ex";
		break;
	default:
		break;
	}
	return "";
}
#endif

IrisInstructorMaker::IrisInstructorMaker()
{
}


IrisInstructorMaker * IrisInstructorMaker::CurrentInstructor()
{
	if (!s_pInstance) {
		s_pInstance = new IrisInstructorMaker();
	}
	return s_pInstance;
}

IrisInstructorMaker::~IrisInstructorMaker()
{
}

void IrisInstructorMaker::push_env()
{
	AddInstructorCode(PUSH_ENV, 0);
}

void IrisInstructorMaker::pop_env()
{
	AddInstructorCode(POP_ENV, 0);
}

void IrisInstructorMaker::push()
{
	AddInstructorCode(PUSH, 0);
}

void IrisInstructorMaker::pop(IR_DWORD dwIndex)
{
	AddInstructorCode(POP, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::cre_env()
{
	AddInstructorCode(CRE_ENV, 0);
}

void IrisInstructorMaker::load(IrisAMType eType, IR_DWORD dwIndex)
{
	AddInstructorCode(LOAD, 1);
	AddAMCode(eType, dwIndex);
}

void IrisInstructorMaker::nol_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddInstructorCode(NOL_CALL, 2);
	AddAMCode(IrisAMType::Identifier, dwIndex);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}

void IrisInstructorMaker::assign(IrisAMType eType, IR_DWORD dwIndex)
{
	AddInstructorCode(ASSIGN, 1);
	AddAMCode(eType, dwIndex);
}

void IrisInstructorMaker::hid_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddInstructorCode(HID_CALL, 2);
	AddAMCode(IrisAMType::Identifier, dwIndex);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}

void IrisInstructorMaker::set_fld(IR_DWORDS& lsFieldMember)
{
	AddInstructorCode(SET_FLD, (IR_BYTE)lsFieldMember.size());
 	for (auto elem : lsFieldMember) {
		AddAMCode(IrisAMType::Identifier, elem);
	}
}

void IrisInstructorMaker::clr_fld()
{
	AddInstructorCode(CLR_FLD, 0);
}

void IrisInstructorMaker::fld_load(IR_DWORD dwIndex, IR_BYTE bEmptyFlag)
{
	AddInstructorCode(FLD_LOAD, 2);
	AddAMCode(IrisAMType::Constance, dwIndex);
	AddAMCode(IrisAMType::Extends, bEmptyFlag);
}

void IrisInstructorMaker::load_nil()
{
	AddInstructorCode(LOAD_NIL, 0);
}

void IrisInstructorMaker::load_true()
{
	AddInstructorCode(LOAD_TRUE, 0);
}

void IrisInstructorMaker::load_false()
{
	AddInstructorCode(LOAD_FALSE, 0);
}

void IrisInstructorMaker::load_self()
{
	AddInstructorCode(LOAD_SELF, 0);
}

void IrisInstructorMaker::imth_def(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(IMTH_DEF, (IR_BYTE)lsParameters.size() + 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);

	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}

	AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
	AddAMCode(IrisAMType::Extends, bWithCastBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::cmth_def(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(CMTH_DEF, (IR_BYTE)lsParameters.size() + 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);

	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}

	AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
	AddAMCode(IrisAMType::Extends, bWithCastBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::blk_def(IR_DWORD dwIndex)
{
	AddInstructorCode(BLK_DEF, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::end_def(IR_DWORD dwIndex)
{
	AddInstructorCode(END_DEF, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::jfon(IR_DWORD dwOffset)
{
	AddInstructorCode(JFON, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::jmp(IR_DWORD dwOffset)
{
	AddInstructorCode(JMP, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::ini_tm()
{
	AddInstructorCode(INI_TM, 0);
}

void IrisInstructorMaker::ini_cnt()
{
	AddInstructorCode(INI_CNT, 0);
}

void IrisInstructorMaker::cmp_tac()
{
	AddInstructorCode(CMP_TAC, 0);
}

void IrisInstructorMaker::inc_cnt()
{
	AddInstructorCode(INC_CNT, 0);
}

void IrisInstructorMaker::assign_log(IR_DWORD dwParameterCount)
{
	AddInstructorCode(ASSIGN_LOG, 1);
	AddAMCode(IrisAMType::LocalValue, dwParameterCount);
}

void IrisInstructorMaker::brk()
{
	AddInstructorCode(BRK, 0);
}

void IrisInstructorMaker::push_deep(IR_DWORD dwParameterCount)
{
	AddInstructorCode(PUSH_DEEP, 1);
	AddAMCode(IrisAMType::Extends, dwParameterCount);
}

void IrisInstructorMaker::pop_deep()
{
	AddInstructorCode(POP_DEEP, 0);
}

void IrisInstructorMaker::rtn()
{
	AddInstructorCode(RTN, 0);
}

void IrisInstructorMaker::ctn()
{
	AddInstructorCode(CTN, 0);
}

void IrisInstructorMaker::assign_vsl()
{
	AddInstructorCode(ASSIGN_VSL, 0);

}

void IrisInstructorMaker::assign_iter()
{
	AddInstructorCode(ASSIGN_ITER, 0);

}

void IrisInstructorMaker::load_iter()
{
	AddInstructorCode(LOAD_ITER, 0);

}

void IrisInstructorMaker::jt(IR_DWORD dwOffset)
{
	AddInstructorCode(JT, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::assign_cmp()
{
	AddInstructorCode(ASSIGN_CMP, 0);
}

void IrisInstructorMaker::cmp_cmp()
{
	AddInstructorCode(CMP_CMP, 0);
}

void IrisInstructorMaker::cre_cenv(IR_BYTE bType)
{
	AddInstructorCode(CRE_CENV, 1);
	AddAMCode(IrisAMType::Extends, bType);
}

void IrisInstructorMaker::def_cls(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(DEF_CLS, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::add_ext()
{
	AddInstructorCode(ADD_EXT, 0);
}

void IrisInstructorMaker::add_mld()
{
	AddInstructorCode(ADD_MLD, 0);
}

void IrisInstructorMaker::add_inf()
{
	AddInstructorCode(ADD_INF, 0);
}

void IrisInstructorMaker::push_cnt()
{
	AddInstructorCode(PUSH_CNT, 0);
}

void IrisInstructorMaker::push_tim()
{
	AddInstructorCode(PUSH_TIM, 0);
}

void IrisInstructorMaker::pop_cnt(IR_BYTE bDeep)
{
	AddInstructorCode(POP_CNT, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::pop_tim(IR_BYTE bDeep)
{
	AddInstructorCode(POP_TIM, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_unim()
{
	AddInstructorCode(PUSH_UNIM, 0);
}

void IrisInstructorMaker::pop_unim(IR_BYTE bDeep)
{
	AddInstructorCode(POP_UNIM, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_vsl()
{
	AddInstructorCode(PUSH_VSL, 0);
}

void IrisInstructorMaker::pop_vsl(IR_BYTE bDeep)
{
	AddInstructorCode(POP_VSL, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_iter()
{
	AddInstructorCode(PUSH_ITER, 0);
}

void IrisInstructorMaker::pop_iter(IR_BYTE bDeep)
{
	AddInstructorCode(POP_ITER, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::str_def(IR_DWORD dwNameIndex, IR_DWORD dwParamIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(STR_DEF, 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Identifier, dwParamIndex);
	AddAMCode(IrisAMType::Extends, bIsWithBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::gtr_def(IR_DWORD dwNameIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(GTR_DEF, 3);	
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, bIsWithBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::gstr_def(IR_DWORD dwNameIndex)
{
	AddInstructorCode(GSTR_DEF, 1);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
}

void IrisInstructorMaker::set_auth(IR_DWORD dwNameIndex, IR_BYTE bEnv, IR_BYTE bTar, IR_BYTE bType)
{
	AddInstructorCode(SET_AUTH, 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, bEnv);
	AddAMCode(IrisAMType::Extends, bTar);
	AddAMCode(IrisAMType::Extends, bType);
}

void IrisInstructorMaker::def_mld(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(DEF_MLD, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::def_inf(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(DEF_INF, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::def_infs(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex)
{
	AddInstructorCode(DEF_INFS, (IR_BYTE)lsParameters.size() + 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);

	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}

	AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
}

void IrisInstructorMaker::cblk_def(IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(CBLK_DEF, (IR_BYTE)lsParameters.size() + 1 + 1);
	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}
	AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::blk()
{
	AddInstructorCode(BLK, 0);
}

//void IrisInstructorMaker::cast(IR_DWORD nParametersCount)
//{
//	AddInstructorCode(62, 1);
//	AddAMCode(IrisAMType::Extends, nParametersCount);
//}

void IrisInstructorMaker::reg_irp(IR_BYTE bWithIgnoreBlock, IR_DWORD nIndex)
{
	AddInstructorCode(REG_IRP, 2);
	AddAMCode(IrisAMType::Extends, bWithIgnoreBlock);
	AddAMCode(IrisAMType::Extends, nIndex);
}

void IrisInstructorMaker::ureg_irp(IR_DWORD nIndex)
{
	AddInstructorCode(UREG_IRP, 1);
	AddAMCode(IrisAMType::Extends, nIndex);
}

void IrisInstructorMaker::assign_ir(IR_DWORD dwIndex)
{
	AddInstructorCode(ASSIGN_IR, 1);
	AddAMCode(IrisAMType::LocalValue, dwIndex);
}

void IrisInstructorMaker::grn()
{
	AddInstructorCode(GRN, 0);
}

void IrisInstructorMaker::spr(IR_BYTE bParameterCount)
{
	AddInstructorCode(SPR, 1);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}

void IrisInstructorMaker::load_cast()
{
	AddInstructorCode(LOAD_CAST, 0);
}
