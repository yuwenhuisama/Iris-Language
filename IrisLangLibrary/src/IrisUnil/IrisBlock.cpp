#include "IrisUnil/IrisBlock.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisInterpreter.h"
#include "IrisValidator/IrisAbstractStatementValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"

IrisBlock::IrisBlock(IrisList<IrisStatement*>* pStatements) : m_pStatements(pStatements)
{
}

bool IrisBlock::Generate()
{

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->IncreamDefineIndex();
	pMaker->blk_def(pCompiler->GetDefineIndex());

	if (m_pStatements) {
		if(!m_pStatements->Ergodic(
			[&](IrisStatement* pStatement) -> bool {
			if(!pStatement->Generate()) {
				return false;
			}
			return true;
		}
			))
			return false;
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();

	return true;
}

IrisBlock::~IrisBlock()
{
	if (m_pStatements) {
		m_pStatements->Ergodic([](IrisStatement*& pStatement) -> bool {
			delete pStatement;
			pStatement = nullptr;
			return true;
		});
	}
}

bool IrisBlock::Accept(IrisAbstractStatementValidateVisitor* pVisitor) {
	return pVisitor->Visit(this);
}

bool IrisBlock::Validate()
{
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (m_pStatements && !m_pStatements->Ergodic([&](IrisStatement*& pStatement) -> bool {
		if (!pStatement->Accept(&isvvStatementVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	return true;
}

