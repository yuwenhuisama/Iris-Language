#include "IrisComponents/IrisExpressions/IrisIndexExpression.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"

IrisIndexExpression::IrisIndexExpression(IrisExpression* pTarget, IrisExpression* pIndexer) : m_pTarget(pTarget), m_pIndexer(pIndexer)
{
}


bool IrisIndexExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (!m_pIndexer->Generate()) {
		return false;
	}
	pMaker->push();

	if (!m_pTarget->Generate()) {
		return false;
	}

	pMaker->load(IrisAMType::IndexValue, 0);

	pMaker->pop(1);

	//pMaker->push_env();
	//pMaker->cre_env();

	//pMaker->nol_call(pCompiler->GetIdentifierIndex("[]"), 1);

	//pMaker->pop_env();
	//pMaker->pop(1);

	return true;
}

bool IrisIndexExpression::LeftValue(IrisAMType & eType, IR_DWORD & bIndex)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push();

	if (!m_pIndexer->Generate()) {
		return false;
	}
	pMaker->push();

	if (!m_pTarget->Generate()) {
		return false;
	}

	eType = IrisAMType::IndexValue;
	bIndex = 0;
	return true;
}

IrisIndexExpression::~IrisIndexExpression()
{
	if (m_pTarget) {
		delete m_pTarget;
	}
	if (m_pIndexer) {
		delete m_pIndexer;
	}
}
