#include "IrisComponents/IrisParts/IrisClosureBlockLiteral.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

bool IrisClosureBlockLiteral::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();

	list<IR_DWORD> lsParameters;

	if (m_pParameters) {
		m_pParameters->Ergodic(
			[&](IrisIdentifier*& pIdentifier) -> bool {
			lsParameters.push_back(pCompiler->GetIdentifierIndex(pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
			return true;
		}
		);
	}

	pCompiler->IncreamDefineIndex();
	pMaker->cblk_def(lsParameters, pCompiler->GetDefineIndex());

	if(m_pStatements) {
		if(!m_pStatements->Ergodic(
			[&](IrisStatement*& pStatement) -> bool {
			if (!pStatement->Generate()) {
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

IrisClosureBlockLiteral::IrisClosureBlockLiteral(IrisList<IrisIdentifier*>* pParameters, IrisList<IrisStatement*>* pStatements) : m_pParameters(pParameters), m_pStatements(pStatements)
{
}


IrisClosureBlockLiteral::~IrisClosureBlockLiteral()
{
	if (m_pParameters) {
		m_pParameters->Ergodic(
			[](IrisIdentifier*& pIdentifier) -> bool { delete pIdentifier; pIdentifier = nullptr; return true; }
		);
	}
	if (m_pStatements) {
		m_pStatements->Ergodic(
			[](IrisStatement*& pStatement) -> bool { delete pStatement; pStatement = nullptr; return true; }
		);
	}
}
