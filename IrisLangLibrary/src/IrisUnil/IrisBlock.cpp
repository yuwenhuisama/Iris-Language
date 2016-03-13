#include "IrisUnil/IrisBlock.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisInterpreter.h"

IrisBlock::IrisBlock(IrisList<IrisStatement*>* pStatements) : m_pStatements(pStatements)
{
}

bool IrisBlock::Generate()
{

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->IncreamDefineIndex();
	pMaker->blk_def(pCompiler->GetDefineIndex());

	//for (auto stmt : m_pStatements->m_lsList) {
	//	stmt->Generate();
	//}

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
}
