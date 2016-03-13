#include "IrisComponents/IrisExpressions/IrisHashExpression.h"
#include "IrisComponents/IrisParts/IrisHashPair.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

bool IrisHashExpression::Generate()
{
	// ²ÎÊýÈëÕ»
	int nCount = 0;
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (m_pHashPairs) {
		if(!m_pHashPairs->Ergodic(
			[&](IrisHashPair*& pPair) -> bool {
			if (!pPair->m_pKey->Generate()) {
				return false;
			}
			pMaker->push();
			++nCount;
			if (!pPair->m_pValue->Generate()) {
				return false;
			}
			pMaker->push();
			++nCount;
			return true;
		}
		))
			return false;
	}

	pMaker->load(IrisAMType::Constance, pCompiler->GetIdentifierIndex("Hash", pCompiler->GetCurrentFileIndex()));
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("new", pCompiler->GetCurrentFileIndex()), nCount);
	pMaker->pop_env();
	if (nCount > 0) {
		pMaker->pop(nCount);
	}

	return true;
}

IrisHashExpression::IrisHashExpression(IrisList<IrisHashPair*>* pHashPairs) : m_pHashPairs(pHashPairs)
{
}


IrisHashExpression::~IrisHashExpression()
{
	if (m_pHashPairs) {
		m_pHashPairs->Ergodic(
			[](IrisHashPair*& pPair) -> bool { delete pPair; pPair = nullptr; return true; }
		);
	}
}
