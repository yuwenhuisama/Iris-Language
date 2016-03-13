#include "IrisComponents/IrisStatements/IrisCastStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

IrisCastStatement::IrisCastStatement(IrisList<IrisExpression*>* pParameters) : m_pParameters(pParameters)
{
}


bool IrisCastStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	list<IR_WORD> lsParameters;
	if (m_pParameters) {
		if(!m_pParameters->Ergodic(
			[&](IrisExpression*& pExpression) -> bool {
			if(!pExpression->Generate())
				return false;
			pMaker->push();
			return true;
		}
		))
			return false;
	}
	pMaker->cast(m_pParameters ? m_pParameters->GetSize() : 0);
	if (m_pParameters) {
		pMaker->pop(m_pParameters->GetSize());
	}
	return true;
}

IrisCastStatement::~IrisCastStatement()
{
	if (m_pParameters) {
		m_pParameters->Ergodic(
			[](IrisExpression*& pExpression) -> bool { delete pExpression; pExpression = nullptr; return true; }
		);
	}
}
