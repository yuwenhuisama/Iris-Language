#include "IrisComponents/IrisExpressions/IrisBinaryExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"

bool IrisBinaryExpression::OperateGenerate(unsigned int nOperatorIndex)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();

	// 先计算右边
	if (!m_pRightExpression->Generate()) {
		return false;
	}
	// 将结果作为参数
	pMaker->push();
	// 再计算左边
	if (!m_pLeftExpression->Generate()) {
		return false;
	}
	// 保存env
	pMaker->push_env();
	// 创建新env
	pMaker->cre_env();
	// call
	pMaker->nol_call(nOperatorIndex, 1);
	// 恢复env
	pMaker->pop_env();
	// 参数出栈
	pMaker->pop(1);

	return true;
}

bool IrisBinaryExpression::AssignGenerate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	
	// 先计算右边
	if (!m_pRightExpression->Generate()) {
		return false;
	}

	IrisAMType eType = IrisAMType::LocalValue;
	IR_DWORD nIndex = 0;
	// 获取左值参数
	if (!m_pLeftExpression->LeftValue(eType, nIndex)) {
		return false;
	}

	// 赋值
	pMaker->assign(eType, nIndex);

	if (eType == IrisAMType::MemberValue) {
		pMaker->pop(1);
	}
	else if (eType == IrisAMType::IndexValue) {
		pMaker->pop(2);
	}

	return true;
}

bool IrisBinaryExpression::OperateAssignGenerate(unsigned int nOperatorIndex)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	
	// 先计算
	if (!OperateGenerate(nOperatorIndex)) {
		return false;
	}
	// 然后赋值
	IrisAMType eType = IrisAMType::LocalValue;
	IR_DWORD nIndex = 0;
	// 获取左值参数
	if (!m_pLeftExpression->LeftValue(eType, nIndex)) {
		return false;
	}

	// 赋值
	pMaker->assign(eType, nIndex);

	if (eType == IrisAMType::MemberValue) {
		pMaker->pop(1);
	}
	else if (eType == IrisAMType::IndexValue) {
		pMaker->pop(2);
	}
	return true;
}

bool IrisBinaryExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	unsigned int nOperatorIndex = 0;
	bool bResult = false;
	pCompiler->SetLineNumber(m_nLineNumber);

	switch (m_eType)
	{
	case IrisBinaryExpressionType::Assign:
		return AssignGenerate();
		break;
	case IrisBinaryExpressionType::AssignAdd:
		nOperatorIndex = pCompiler->GetIdentifierIndex("+", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignSub:
		nOperatorIndex = pCompiler->GetIdentifierIndex("-", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignMul:
		nOperatorIndex = pCompiler->GetIdentifierIndex("*", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignDiv:
		nOperatorIndex = pCompiler->GetIdentifierIndex("/", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignMod:
		nOperatorIndex = pCompiler->GetIdentifierIndex("%", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitAnd:
		nOperatorIndex = pCompiler->GetIdentifierIndex("&", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitOr:
		nOperatorIndex = pCompiler->GetIdentifierIndex("|", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitXor:
		nOperatorIndex = pCompiler->GetIdentifierIndex("^", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitShr:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">>", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitShl:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<<", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitSar:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">>>", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::AssignBitSal:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<<<", pCompiler->GetCurrentFileIndex());
		bResult = OperateAssignGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LogicOr:
		nOperatorIndex = pCompiler->GetIdentifierIndex("||", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LogicAnd:
		nOperatorIndex = pCompiler->GetIdentifierIndex("&&", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LogicBitOr:
		nOperatorIndex = pCompiler->GetIdentifierIndex("|", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LogicBitXor:
		nOperatorIndex = pCompiler->GetIdentifierIndex("^", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LogicBitAnd:
		nOperatorIndex = pCompiler->GetIdentifierIndex("&", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Equal:
		nOperatorIndex = pCompiler->GetIdentifierIndex("==", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::NotEqual:
		nOperatorIndex = pCompiler->GetIdentifierIndex("!=", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::GreatThan:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::GreatThanOrEqual:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">=", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LessThan:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::LessThanOrEqual:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<=", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::BitShr:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">>", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::BitShl:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<<", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::BitSar:
		nOperatorIndex = pCompiler->GetIdentifierIndex(">>>", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::BitSal:
		nOperatorIndex = pCompiler->GetIdentifierIndex("<<<", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Add:
		nOperatorIndex = pCompiler->GetIdentifierIndex("+", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Sub:
		nOperatorIndex = pCompiler->GetIdentifierIndex("-", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Mul:
		nOperatorIndex = pCompiler->GetIdentifierIndex("*", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Div:
		nOperatorIndex = pCompiler->GetIdentifierIndex("/", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Mod:
		nOperatorIndex = pCompiler->GetIdentifierIndex("%", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	case IrisBinaryExpressionType::Power:
		nOperatorIndex = pCompiler->GetIdentifierIndex("**", pCompiler->GetCurrentFileIndex());
		bResult = OperateGenerate(nOperatorIndex);
		break;
	default:
		break;
	}

	return bResult;
}

IrisBinaryExpression::IrisBinaryExpression(IrisBinaryExpressionType eType, IrisExpression * pLeftExpression, IrisExpression * pRightExpression) : m_eType(eType), m_pLeftExpression(pLeftExpression), m_pRightExpression(pRightExpression)
{
}

IrisBinaryExpression::~IrisBinaryExpression()
{
	if (m_pLeftExpression)
		delete m_pLeftExpression;
	if (m_pRightExpression)
		delete m_pRightExpression;
}
