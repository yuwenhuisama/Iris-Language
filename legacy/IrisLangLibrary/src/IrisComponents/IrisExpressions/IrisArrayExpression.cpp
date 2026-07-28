#include "IrisComponents/IrisExpressions/IrisArrayExpression.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"

bool IrisArrayExpression::Generate()
{
	// ²ÎÊýÈëÕ»
	int nCount = 0;
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();

	pCompiler->SetLineNumber(m_nLineNumber);

	if (m_pElementList) {
		if(!m_pElementList->Ergodic(
			[&](IrisExpression*& pExpression) -> bool {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->push();
			++nCount;
			return true;
		}
		))
			return false;
	}

	pMaker->load(IrisAMType::Constance, pCompiler->GetIdentifierIndex("Array", pCompiler->GetCurrentFileIndex()));
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("new", pCompiler->GetCurrentFileIndex()), nCount);
	pMaker->pop_env();
	if (nCount > 0) {
		pMaker->pop(nCount);
	}

	return true;
}

IrisArrayExpression::IrisArrayExpression(IrisList<IrisExpression*>* pElementList) : m_pElementList(pElementList)
{
}


IrisArrayExpression::~IrisArrayExpression()
{
	if (m_pElementList) {
		m_pElementList->Ergodic(
			[](IrisExpression*& pExpression) ->bool { delete pExpression; pExpression = nullptr; return true; }
		);
		delete m_pElementList;
	}

}



bool IrisArrayExpression::Validate()
{
	IrisExpressionValidateVisitor ievvExpressionVisitor;

	if (m_pElementList && !m_pElementList->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	return true;
}
