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

void IrisInstructorMaker::Generate()
{
	for (auto& pfGenerate : m_lsGenerateInstructionList) {
		pfGenerate();
	}
}

void IrisInstructorMaker::place_lable(Label * pLabel)
{
	pLabel->SetStartOffset(IrisCompiler::CurrentCompiler()->GetCurrentCodePosition() + 2);
}

void IrisInstructorMaker::push_env()
{
	AddGenerateInstruction([=]() -> void{
		AddInstructorCode(PUSH_ENV, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop_env()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_ENV, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::push()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop(IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP, 1);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::cre_env()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CRE_ENV, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::load(IrisAMType eType, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD, 1);
		AddAMCode(eType, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::nol_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(NOL_CALL, 2);
		AddAMCode(IrisAMType::Identifier, dwIndex);
		AddAMCode(IrisAMType::Extends, bParameterCount);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::assign(IrisAMType eType, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN, 1);
		AddAMCode(eType, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::hid_call(IR_DWORD dwIndex, IR_BYTE bParameterCount)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(HID_CALL, 2);
		AddAMCode(IrisAMType::Identifier, dwIndex);
		AddAMCode(IrisAMType::Extends, bParameterCount);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::set_fld(IR_DWORDS& lsFieldMember)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(SET_FLD, (IR_BYTE)lsFieldMember.size());
		for (auto elem : lsFieldMember) {
			AddAMCode(IrisAMType::Identifier, elem);
		}
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(lsFieldMember.size()));
}

void IrisInstructorMaker::clr_fld()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CLR_FLD, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::fld_load(IR_DWORD dwIndex, IR_BYTE bEmptyFlag)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(FLD_LOAD, 2);
		AddAMCode(IrisAMType::Constance, dwIndex);
		AddAMCode(IrisAMType::Extends, bEmptyFlag);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::load_nil()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_NIL, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::load_true()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_TRUE, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::load_false()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_FALSE, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::load_self()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_SELF, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::imth_def(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(IMTH_DEF, (IR_BYTE)lsParameters.size() + 4);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);

		for (auto elem : lsParameters) {
			AddAMCode(IrisAMType::Identifier, elem);
		}

		AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
		AddAMCode(IrisAMType::Extends, bWithCastBlock);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(lsParameters.size() + 4));
}

void IrisInstructorMaker::cmth_def(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_BYTE bWithCastBlock, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CMTH_DEF, (IR_BYTE)lsParameters.size() + 4);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);

		for (auto elem : lsParameters) {
			AddAMCode(IrisAMType::Identifier, elem);
		}

		AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
		AddAMCode(IrisAMType::Extends, bWithCastBlock);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});

	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(lsParameters.size() + 4));
}

void IrisInstructorMaker::blk_def(IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(BLK_DEF, 1);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::end_def(IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(END_DEF, 1);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::jfon(Label* pLabel)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(JFON, 1);
		AddAMCode(IrisAMType::Extends, pLabel->GetStartOffset() - IrisCompiler::CurrentCompiler()->GetCurrentCodeLastIndex());
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::jmp(Label* pLabel)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(JMP, 1);
		AddAMCode(IrisAMType::Extends, pLabel->GetStartOffset() - IrisCompiler::CurrentCompiler()->GetCurrentCodeLastIndex());
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::ini_tm()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(INI_TM, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::ini_cnt()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(INI_CNT, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::cmp_tac()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CMP_TAC, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::inc_cnt()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(INC_CNT, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::assign_log(IR_DWORD dwParameterCount)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN_LOG, 1);
		AddAMCode(IrisAMType::LocalValue, dwParameterCount);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::brk(Label* pLabel)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(BRK, 1);
		AddAMCode(IrisAMType::Extends, pLabel->GetStartOffset() - IrisCompiler::CurrentCompiler()->GetCurrentCodeLastIndex());
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::push_deep(IR_DWORD dwParameterCount)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_DEEP, 1);
		AddAMCode(IrisAMType::Extends, dwParameterCount);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::pop_deep()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_DEEP, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::rtn()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(RTN, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::ctn(Label* pLabel)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CTN, 1);
		AddAMCode(IrisAMType::Extends, pLabel->GetStartOffset() - IrisCompiler::CurrentCompiler()->GetCurrentCodeLastIndex());
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::assign_vsl()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN_VSL, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::assign_iter()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN_ITER, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::load_iter()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_ITER, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::jt(Label* pLabel)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(JT, 1);
		AddAMCode(IrisAMType::Extends, pLabel->GetStartOffset() - IrisCompiler::CurrentCompiler()->GetCurrentCodeLastIndex());
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::assign_cmp()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN_CMP, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::cmp_cmp()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CMP_CMP, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::cre_cenv(IR_BYTE bType)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CRE_CENV, 1);
		AddAMCode(IrisAMType::Extends, bType);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::def_cls(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(DEF_CLS, 2);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::add_ext()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ADD_EXT, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::add_mld()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ADD_MLD, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::add_inf()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ADD_INF, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::push_cnt()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_CNT, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::push_tim()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_TIM, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop_cnt(IR_BYTE bDeep)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_CNT, 1);
		AddAMCode(IrisAMType::Extends, bDeep);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::pop_tim(IR_BYTE bDeep)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_TIM, 1);
		AddAMCode(IrisAMType::Extends, bDeep);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::push_unim()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_UNIM, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop_unim(IR_BYTE bDeep)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_UNIM, 1);
		AddAMCode(IrisAMType::Extends, bDeep);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::push_vsl()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_VSL, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop_vsl(IR_BYTE bDeep)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_VSL, 1);
		AddAMCode(IrisAMType::Extends, bDeep);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::push_iter()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(PUSH_ITER, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::pop_iter(IR_BYTE bDeep)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(POP_ITER, 1);
		AddAMCode(IrisAMType::Extends, bDeep);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::str_def(IR_DWORD dwNameIndex, IR_DWORD dwParamIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(STR_DEF, 4);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Identifier, dwParamIndex);
		AddAMCode(IrisAMType::Extends, bIsWithBlock);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(4));
}

void IrisInstructorMaker::gtr_def(IR_DWORD dwNameIndex, IR_BYTE bIsWithBlock, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(GTR_DEF, 3);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Extends, bIsWithBlock);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(3));
}

void IrisInstructorMaker::gstr_def(IR_DWORD dwNameIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(GSTR_DEF, 1);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::set_auth(IR_DWORD dwNameIndex, IR_BYTE bEnv, IR_BYTE bTar, IR_BYTE bType)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(SET_AUTH, 4);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Extends, bEnv);
		AddAMCode(IrisAMType::Extends, bTar);
		AddAMCode(IrisAMType::Extends, bType);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(4));
}

void IrisInstructorMaker::def_mld(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(DEF_MLD, 2);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::def_inf(IR_DWORD dwNameIndex, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(DEF_INF, 2);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::def_infs(IR_DWORD dwNameIndex, IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(DEF_INFS, (IR_BYTE)lsParameters.size() + 2);
		AddAMCode(IrisAMType::Identifier, dwNameIndex);

		for (auto elem : lsParameters) {
			AddAMCode(IrisAMType::Identifier, elem);
		}

		AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(lsParameters.size() + 2));
}

void IrisInstructorMaker::cblk_def(IR_DWORDS & lsParameters, IR_DWORD dwVariableParameterIndex, IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(CBLK_DEF, (IR_BYTE)lsParameters.size() + 2);
		for (auto elem : lsParameters) {
			AddAMCode(IrisAMType::Identifier, elem);
		}
		AddAMCode(IrisAMType::Identifier, dwVariableParameterIndex);
		AddAMCode(IrisAMType::Extends, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(lsParameters.size() + 2));
}

void IrisInstructorMaker::blk()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(BLK, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::reg_irp(IR_BYTE bWithIgnoreBlock, IR_DWORD nIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(REG_IRP, 2);
		AddAMCode(IrisAMType::Extends, bWithIgnoreBlock);
		AddAMCode(IrisAMType::Extends, nIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(2));
}

void IrisInstructorMaker::ureg_irp(IR_DWORD nIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(UREG_IRP, 1);
		AddAMCode(IrisAMType::Extends, nIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::assign_ir(IR_DWORD dwIndex)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(ASSIGN_IR, 1);
		AddAMCode(IrisAMType::LocalValue, dwIndex);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::grn()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(GRN, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::spr(IR_BYTE bParameterCount)
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(SPR, 1);
		AddAMCode(IrisAMType::Extends, bParameterCount);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(1));
}

void IrisInstructorMaker::load_cast()
{
	AddGenerateInstruction([=]() -> void {
		AddInstructorCode(LOAD_CAST, 0);
	});
	IrisCompiler::CurrentCompiler()->IncreamCurrentCodePoisition(CalcInstructorLength(0));
}

void IrisInstructorMaker::Label::SetStartOffset(size_t nOffset) { m_nStartOffset = nOffset; }

size_t IrisInstructorMaker::Label::GetStartOffset() { return m_nStartOffset; }
