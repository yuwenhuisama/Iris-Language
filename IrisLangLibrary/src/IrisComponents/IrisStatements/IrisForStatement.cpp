#include "IrisComponents/IrisStatements/IrisForStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"

#include "IrisValidator/IrisStatementValidateVisitor.h"

#include "IrisThread/IrisThreadManager.h"

#include <vector>
using namespace std;

bool IrisForStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push_vsl();
	pMaker->push_iter();
	if (!m_pSource->Generate()) {
		return false;
	}
	pMaker->assign_vsl();
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("get_iterator", pCompiler->GetCurrentFileIndex()), 0);
	pMaker->pop_env();
	pMaker->assign_iter();
	pMaker->push_deep(pCompiler->GetDefineIndex() + 1);

	vector<IR_WORD> vcBefore;
	vector<IR_WORD> vcAfter;
	vector<IR_WORD> vcEndBlk;
	vector<IR_WORD>* pOldVector = pCompiler->GetCurrentCodeVector();
	pCompiler->SetCurrentCodeList(&vcBefore);

	// next
	pMaker->load_iter();
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("next", pCompiler->GetCurrentFileIndex()), 0);
	pMaker->pop_env();
	// size = 2 + 2 + 2 + 2 + 3 + 3 + 2

	// is end
	pMaker->load_iter();
	pMaker->push_env();
	pMaker->cre_env();
	pMaker->nol_call(pCompiler->GetIdentifierIndex("is_end", pCompiler->GetCurrentFileIndex()), 0);
	pMaker->pop_env();

	pCompiler->SetCurrentCodeList(&vcAfter);
	// assign
	if (m_pIter1 && !m_pIter2) {
		pMaker->load_iter();
		pMaker->push_env();
		pMaker->cre_env();
		pMaker->nol_call(pCompiler->GetIdentifierIndex("get_value", pCompiler->GetCurrentFileIndex()), 0);
		pMaker->pop_env();
		pMaker->assign(IrisAMType::LocalValue, pCompiler->GetIdentifierIndex(m_pIter1->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
	}
	else if(m_pIter1 && m_pIter2){
		pMaker->load_iter();
		pMaker->push_env();
		pMaker->cre_env();
		pMaker->nol_call(pCompiler->GetIdentifierIndex("get_key", pCompiler->GetCurrentFileIndex()), 0);
		pMaker->pop_env();
		pMaker->assign(IrisAMType::LocalValue, pCompiler->GetIdentifierIndex(m_pIter1->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));

		pMaker->load_iter();
		pMaker->push_env();
		pMaker->cre_env();
		pMaker->nol_call(pCompiler->GetIdentifierIndex("get_value", pCompiler->GetCurrentFileIndex()), 0);
		pMaker->pop_env();
		pMaker->assign(IrisAMType::LocalValue, pCompiler->GetIdentifierIndex(m_pIter2->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
	}
	// block
	if (!m_pBlock->Generate()) {
		return false;
	}

	for (int i = 0; i < 4 + 1; ++i) {
		//vcEndBlk.push_front(vcAfter.back());
		vcEndBlk.insert(vcEndBlk.begin(), vcAfter.back());
		vcAfter.pop_back();
	}

	// ¼ÆËãjtºÍjmpµÄoffset
	unsigned int nJtOffset = vcAfter.size() + 4 + 1; // after vector code size + jmp
	int nJmpOffset = vcAfter.size() + 4 + 1 + vcBefore.size() + 4 + 1; // after vector code size + jt + before vector code size + jmp

	int nSkipOffset = 2 + 2 + 2 + 2 + 3 + 3 + 2;

	pCompiler->SetCurrentCodeList(pOldVector);
	pMaker->jmp(nSkipOffset);
	pCompiler->LinkCodesToRealCodes(vcBefore);
	pMaker->jt(nJtOffset);
	pCompiler->LinkCodesToRealCodes(vcAfter);
	pMaker->jmp(-nJmpOffset);
	pCompiler->LinkCodesToRealCodes(vcEndBlk);
	pMaker->pop_vsl(pCompiler->GetDefineIndex() + 1);
	pMaker->pop_iter(pCompiler->GetDefineIndex() + 1);
	pMaker->pop_deep();
	return true;
}

IrisForStatement::IrisForStatement(IrisIdentifier* pIter1, IrisIdentifier* pIter2, IrisExpression* pSource, IrisBlock* pBlock) : m_pIter1(pIter1), m_pIter2(pIter2), m_pSource(pSource), m_pBlock(pBlock)
{
}


IrisForStatement::~IrisForStatement()
{
	if (m_pIter1)
		delete m_pIter1;
	if (m_pIter2)
		delete m_pIter2;
	if (m_pSource)
		delete m_pSource;
	if (m_pBlock)
		delete m_pBlock;
}

bool IrisForStatement::Validate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();

	if (m_pIter1 && !m_pIter2) {
		if (m_pIter1->GetType() != IrisIdentifierType::LocalVariable) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pIter1->GetIdentifierString() + " is not a CONSTANCE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
			return false;
		}
	}
	else if (m_pIter1 && m_pIter2) {
		if (m_pIter1->GetType() != IrisIdentifierType::LocalVariable) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pIter1->GetIdentifierString() + " is not a LOCAL VARIABLE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
			return false;
		}
		else if (m_pIter2->GetType() != IrisIdentifierType::LocalVariable) {
			IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pIter2->GetIdentifierString() + " is not a LOCAL VARIABLE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
			return false;
		}
	}

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::LoopBlock);

		IrisStatementValidateVisitor isvvStatementVisitor;

		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}

	return true;
}
