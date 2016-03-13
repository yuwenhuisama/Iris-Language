#include "IrisComponents/IrisExpressions/IrisRangeExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"


bool IrisRangeExpression::Generate()
{

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	switch (m_eType)
	{
	case IrisRangeType::LeftOpenAndRightOpen:
		pMaker->load(IrisAMType::ImmediateInteger, pCompiler->GetIntegerIndex(static_cast<int>(IrisRangeType::LeftOpenAndRightOpen), pCompiler->GetCurrentFileIndex()));
		break;
	case IrisRangeType::LeftOpenAndRightClosed:
		pMaker->load(IrisAMType::ImmediateInteger, pCompiler->GetIntegerIndex(static_cast<int>(IrisRangeType::LeftOpenAndRightClosed), pCompiler->GetCurrentFileIndex()));
		break;
	case IrisRangeType::LeftClosedAndRightClosed:
		pMaker->load(IrisAMType::ImmediateInteger, pCompiler->GetIntegerIndex(static_cast<int>(IrisRangeType::LeftClosedAndRightClosed), pCompiler->GetCurrentFileIndex()));
		break;
	case IrisRangeType::LeftClosedAndRightOpen:
		pMaker->load(IrisAMType::ImmediateInteger, pCompiler->GetIntegerIndex(static_cast<int>(IrisRangeType::LeftClosedAndRightOpen), pCompiler->GetCurrentFileIndex()));
		break;
	default:
		break;
	}
	pMaker->push();
	
	if (!m_pLeftExpression->Generate()) {
		return false;
	}
	pMaker->push();
	if (!m_pRightExpression->Generate()) {
		return false;
	}
	pMaker->push();
	
	pMaker->load(IrisAMType::Constance, pCompiler->GetIdentifierIndex("Range", pCompiler->GetCurrentFileIndex()));
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("new", pCompiler->GetCurrentFileIndex()), 3);
	pMaker->pop_env();

	pMaker->pop(3);

	return true;
}

IrisRangeExpression::IrisRangeExpression(IrisRangeType eType, IrisExpression* pLeftExpression, IrisExpression* pRightExpression)
	: m_eType(eType), m_pLeftExpression(pLeftExpression), m_pRightExpression(pRightExpression)
{
}

IrisRangeExpression::~IrisRangeExpression()
{
}
