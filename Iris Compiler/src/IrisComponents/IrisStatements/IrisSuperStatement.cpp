#include "IrisComponents/IrisStatements/IrisSuperStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"


bool IrisSuperStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (m_pParameters) {
		if(!m_pParameters->Ergodic(
			[&](IrisExpression*& pExpression) {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->push();
			return true;
		}
		))
			return true;
	}

	pMaker->push_env();
	pMaker->cre_env();
	pMaker->spr(m_pParameters->GetSize());
	pMaker->pop_env();
	if (m_pParameters) {
		pMaker->pop(m_pParameters->GetSize());
	}

	return true;
}

IrisSuperStatement::IrisSuperStatement(IrisList<IrisExpression*>* pParameters) : m_pParameters(pParameters)
{
}

IrisSuperStatement::~IrisSuperStatement()
{
	if (m_pParameters) {
		m_pParameters->Ergodic(
			[](IrisExpression*& pExpression) { delete pExpression; pExpression = nullptr; return true; }
		);
	}
}
