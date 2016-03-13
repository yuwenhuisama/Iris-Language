#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

#ifdef IR_DEBUG_PRINT
#include <iostream>
#include <iomanip>
using namespace std;
#endif

IrisInstructorMaker* IrisInstructorMaker::s_pInstance = nullptr;

#ifdef IR_DEBUG_PRINT
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
	case IrisAMType::SelfMemberValue:
		return "sv";
		break;
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
	AddInstructorCode(0, 0);
}

void IrisInstructorMaker::pop_env()
{
	AddInstructorCode(1, 0);
}

void IrisInstructorMaker::push()
{
	AddInstructorCode(2, 0);
}

void IrisInstructorMaker::pop(IR_DWORD dwIndex)
{
	AddInstructorCode(3, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::cre_env()
{
	AddInstructorCode(4, 0);
}

void IrisInstructorMaker::load(IrisAMType eType, IR_DWORD dwIndex)
{
	AddInstructorCode(5, 1);
	AddAMCode(eType, dwIndex);
}

void IrisInstructorMaker::nol_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddInstructorCode(6, 2);
	AddAMCode(IrisAMType::Identifier, dwIndex);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}

void IrisInstructorMaker::assign(IrisAMType eType, IR_DWORD dwIndex)
{
	AddInstructorCode(7, 1);
	AddAMCode(eType, dwIndex);
}

void IrisInstructorMaker::hid_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddInstructorCode(8, 2);
	AddAMCode(IrisAMType::Identifier, dwIndex);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}

void IrisInstructorMaker::set_fld(IR_DWORDS& lsFieldMember)
{
	AddInstructorCode(9, (IR_BYTE)lsFieldMember.size());
 	for (auto elem : lsFieldMember) {
		AddAMCode(IrisAMType::Identifier, elem);
	}
}

void IrisInstructorMaker::clr_fld()
{
	AddInstructorCode(10, 0);
}

void IrisInstructorMaker::fld_load(IR_DWORD dwIndex, IR_BYTE bEmptyFlag)
{
	AddInstructorCode(11, 2);
	AddAMCode(IrisAMType::Constance, dwIndex);
	AddAMCode(IrisAMType::Extends, bEmptyFlag);
}

void IrisInstructorMaker::load_nil()
{
	AddInstructorCode(12, 0);
}

void IrisInstructorMaker::load_true()
{
	AddInstructorCode(13, 0);
}

void IrisInstructorMaker::load_false()
{
	AddInstructorCode(14, 0);
}

void IrisInstructorMaker::load_self()
{
	AddInstructorCode(15, 0);
}

void IrisInstructorMaker::imth_def(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(16, (IR_BYTE)lsParameters.size() + 4);
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
	AddInstructorCode(17, (IR_BYTE)lsParameters.size() + 4);
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
	AddInstructorCode(18, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::end_def(IR_DWORD dwIndex)
{
	AddInstructorCode(19, 1);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::jfon(IR_DWORD dwOffset)
{
	AddInstructorCode(20, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::jmp(IR_DWORD dwOffset)
{
	AddInstructorCode(21, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::ini_tm()
{
	AddInstructorCode(22, 0);
}

void IrisInstructorMaker::ini_cnt()
{
	AddInstructorCode(23, 0);
}

void IrisInstructorMaker::cmp_tac()
{
	AddInstructorCode(24, 0);
}

void IrisInstructorMaker::inc_cnt()
{
	AddInstructorCode(25, 0);
}

void IrisInstructorMaker::assign_log(IR_DWORD dwParameterCount)
{
	AddInstructorCode(26, 1);
	AddAMCode(IrisAMType::LocalValue, dwParameterCount);
}

void IrisInstructorMaker::brk()
{
	AddInstructorCode(27, 0);
}

void IrisInstructorMaker::push_deep(IR_DWORD dwParameterCount)
{
	AddInstructorCode(28, 1);
	AddAMCode(IrisAMType::Extends, dwParameterCount);
}

void IrisInstructorMaker::pop_deep()
{
	AddInstructorCode(29, 0);
}

void IrisInstructorMaker::rtn()
{
	AddInstructorCode(30, 0);
}

void IrisInstructorMaker::ctn()
{
	AddInstructorCode(31, 0);
}

void IrisInstructorMaker::assign_vsl()
{
	AddInstructorCode(32, 0);

}

void IrisInstructorMaker::assign_iter()
{
	AddInstructorCode(33, 0);

}

void IrisInstructorMaker::load_iter()
{
	AddInstructorCode(34, 0);

}

void IrisInstructorMaker::jt(IR_DWORD dwOffset)
{
	AddInstructorCode(35, 1);
	AddAMCode(IrisAMType::Extends, dwOffset);
}

void IrisInstructorMaker::assign_cmp()
{
	AddInstructorCode(36, 0);
}

void IrisInstructorMaker::cmp_cmp()
{
	AddInstructorCode(37, 0);
}

void IrisInstructorMaker::cre_cenv(IR_BYTE bType)
{
	AddInstructorCode(38, 1);
	AddAMCode(IrisAMType::Extends, bType);
}

void IrisInstructorMaker::def_cls(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(39, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::add_ext()
{
	AddInstructorCode(40, 0);
}

void IrisInstructorMaker::add_mld()
{
	AddInstructorCode(41, 0);
}

void IrisInstructorMaker::add_inf()
{
	AddInstructorCode(42, 0);
}

void IrisInstructorMaker::push_cnt()
{
	AddInstructorCode(43, 0);
}

void IrisInstructorMaker::push_tim()
{
	AddInstructorCode(44, 0);
}

void IrisInstructorMaker::pop_cnt(IR_BYTE bDeep)
{
	AddInstructorCode(45, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::pop_tim(IR_BYTE bDeep)
{
	AddInstructorCode(46, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_unim()
{
	AddInstructorCode(47, 0);
}

void IrisInstructorMaker::pop_unim(IR_BYTE bDeep)
{
	AddInstructorCode(48, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_vsl()
{
	AddInstructorCode(49, 0);
}

void IrisInstructorMaker::pop_vsl(IR_BYTE bDeep)
{
	AddInstructorCode(50, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::push_iter()
{
	AddInstructorCode(51, 0);
}

void IrisInstructorMaker::pop_iter(IR_BYTE bDeep)
{
	AddInstructorCode(52, 1);
	AddAMCode(IrisAMType::Extends, bDeep);
}

void IrisInstructorMaker::str_def(IR_DWORD dwNameIndex, IR_DWORD dwParamIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(53, 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Identifier, dwParamIndex);
	AddAMCode(IrisAMType::Extends, bIsWithBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::gtr_def(IR_DWORD dwNameIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddInstructorCode(54, 3);	
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, bIsWithBlock);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::gstr_def(IR_DWORD dwNameIndex)
{
	AddInstructorCode(55, 1);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
}

void IrisInstructorMaker::set_auth(IR_DWORD dwNameIndex, IR_BYTE bEnv, IR_BYTE bTar, IR_BYTE bType)
{
	AddInstructorCode(56, 4);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, bEnv);
	AddAMCode(IrisAMType::Extends, bTar);
	AddAMCode(IrisAMType::Extends, bType);
}

void IrisInstructorMaker::def_mld(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(57, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::def_inf(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddInstructorCode(58, 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::def_infs(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex)
{
	AddInstructorCode(59, (IR_BYTE)lsParameters.size() + 2);
	AddAMCode(IrisAMType::Identifier, dwNameIndex);

	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}

	AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
}

void IrisInstructorMaker::cblk_def(IR_DWORDS & lsParameters, IR_DWORD dwIndex)
{
	AddInstructorCode(60, (IR_BYTE)lsParameters.size() + 1);
	for (auto elem : lsParameters) {
		AddAMCode(IrisAMType::Identifier, elem);
	}
	AddAMCode(IrisAMType::Extends, dwIndex);
}

void IrisInstructorMaker::blk()
{
	AddInstructorCode(61, 0);
}

void IrisInstructorMaker::cast(IR_DWORD nParametersCount)
{
	AddInstructorCode(62, 1);
	AddAMCode(IrisAMType::Extends, nParametersCount);
}

void IrisInstructorMaker::reg_irp(IR_BYTE bWithIgnoreBlock, IR_DWORD nIndex)
{
	AddInstructorCode(63, 2);
	AddAMCode(IrisAMType::Extends, bWithIgnoreBlock);
	AddAMCode(IrisAMType::Extends, nIndex);
}

void IrisInstructorMaker::ureg_irp(IR_DWORD nIndex)
{
	AddInstructorCode(64, 1);
	AddAMCode(IrisAMType::Extends, nIndex);
}

void IrisInstructorMaker::assign_ir(IR_DWORD dwIndex)
{
	AddInstructorCode(65, 1);
	AddAMCode(IrisAMType::LocalValue, dwIndex);
}

void IrisInstructorMaker::grn()
{
	AddInstructorCode(66, 0);
}

void IrisInstructorMaker::spr(IR_BYTE bParameterCount)
{
	AddInstructorCode(67, 1);
	AddAMCode(IrisAMType::Extends, bParameterCount);
}
