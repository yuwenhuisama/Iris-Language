#include "IrisComponents/IrisStatements/IrisLoopIfStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"

#include "IrisThread/IrisThreadManager.h"

#include <vector>
using namespace std;

#if IR_DEBUG_PRINT
#include <iostream>
#include <iomanip>
#endif


bool IrisLoopIfStatement::Generate() {

	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	auto pLabelJumpToEnd = new IrisInstructorMaker::Label();
	auto pLabelJumpBack = new IrisInstructorMaker::Label();

	// Timer
	if (!m_pLoopTime->Generate()) {
		return false;
	}
	pMaker->push_cnt();
	pMaker->push_tim();
	pMaker->push_unim();
	
	pMaker->ini_tm();
	// Counter
	pMaker->ini_cnt();

	pMaker->push_deep(pCompiler->GetDefineIndex() + 1);

	pMaker->place_lable(pLabelJumpBack);

	// Condition
	if (!m_pCondition->Generate()) {
		return false;
	}

	pMaker->jfon(pLabelJumpToEnd);
	pMaker->cmp_tac();

	pMaker->jfon(pLabelJumpToEnd);

	if (m_pLogVariable) {
		pMaker->assign_log(pCompiler->GetIdentifierIndex(m_pLogVariable->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
	}
	pMaker->inc_cnt();

	IrisInstructorMaker::Label* pOldLoopEndLable = static_cast<IrisInstructorMaker::Label* >(pCompiler->GetCurrentLoopEndLabel());
	pCompiler->SetCurrentLoopEndLable(pLabelJumpToEnd);

	if (!m_pBlock->Generate()) {
		return false;
	}

	pCompiler->SetCurrentLoopEndLable(pOldLoopEndLable);

	pMaker->jmp(pLabelJumpBack);

	pMaker->place_lable(pLabelJumpToEnd);

	pMaker->pop_cnt(pCompiler->GetDefineIndex() + 1);
	pMaker->pop_tim(pCompiler->GetDefineIndex() + 1);
	pMaker->pop_unim(pCompiler->GetDefineIndex() + 1);
	pMaker->pop_deep();

	return true;
}

//bool IrisLoopIfStatement::Generate()
//{
//	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
//	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
//	pCompiler->SetLineNumber(m_nLineNumber);
//
//	vector<IR_WORD> vcNewVector;
//	vector<IR_WORD>* pOldVector = pCompiler->GetCurrentCodeVector();
//
//	// Timer
//	if (!m_pLoopTime->Generate()) {
//		return false;
//	}
//	pMaker->push_cnt();
//	pMaker->push_tim();
//	pMaker->push_unim();
//
//	pMaker->ini_tm();
//	// Counter
//	pMaker->ini_cnt();
//
//	pCompiler->SetCurrentCodeList(&vcNewVector);
//
//	vector<IR_WORD> vcConditionCodes;
//	vector<IR_WORD> vcBlockCodes;
//
//	pMaker->push_deep(pCompiler->GetDefineIndex() + 1);
//	// Condition
//	if (!m_pCondition->Generate()) {
//		return false;
//	}
//	vcConditionCodes.assign(vcNewVector.begin(), vcNewVector.end());
//
//	// Block
//	vcNewVector.clear();
//
//	if (m_pLogVariable) {
//		pMaker->assign_log(pCompiler->GetIdentifierIndex(m_pLogVariable->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
//	}
//	pMaker->inc_cnt();
//	if (!m_pBlock->Generate()) {
//		return false;
//	}
//
//	vector<IR_WORD> lsEndBlk;
//
//	for (int i = 0; i < 4 + 1; ++i) {
//		//lsEndBlk.push_front(vcNewVector.back());
//		lsEndBlk.insert(lsEndBlk.begin(), vcNewVector.back());
//		vcNewVector.pop_back();
//	}
//
//	vcBlockCodes.assign(vcNewVector.begin(), vcNewVector.end());
//
//	pCompiler->SetCurrentCodeList(pOldVector);
//
//	unsigned int nConditionJfonOffset = 1 + 1 + 4 + 1 + vcBlockCodes.size() + 4 + 1 + 4 + 1; // cmp_tac + jfon + block codes + jmp + end_blk
//	unsigned int nTimeJfonOffset = vcBlockCodes.size() + 4 + 1 + 4 + 1; // block codes + jmp + end_blk
//	int nJmpOffset = vcBlockCodes.size() + 1 + 1 + 4 + 1 + 4 + 1 + vcConditionCodes.size() + 4 + 1 - 4 - 1; // block size + cmp_tac + jfon + jfon + condition codes + jmp itself - push_deep
//
//	pCompiler->LinkCodesToRealCodes(vcConditionCodes);
//
//	pMaker->jfon(nConditionJfonOffset);
//	pMaker->cmp_tac();
//	pMaker->jfon(nTimeJfonOffset);
//	pCompiler->LinkCodesToRealCodes(vcBlockCodes);
//
//	pMaker->jmp(-nJmpOffset);
//	pCompiler->LinkCodesToRealCodes(lsEndBlk);
//	pMaker->pop_cnt(pCompiler->GetDefineIndex() + 1);
//	pMaker->pop_tim(pCompiler->GetDefineIndex() + 1);
//	pMaker->pop_unim(pCompiler->GetDefineIndex() + 1);
//	pMaker->pop_deep();
//	
//	return true;
//}

IrisLoopIfStatement::IrisLoopIfStatement(IrisExpression* pCondition, IrisExpression* pLoopTime, IrisIdentifier* pLogVariable, IrisBlock* pBlock) : m_pCondition(pCondition), m_pLoopTime(pLoopTime), m_pLogVariable(pLogVariable), m_pBlock(pBlock)
{
}


IrisLoopIfStatement::~IrisLoopIfStatement()
{
	if (m_pCondition)
		delete m_pCondition;
	if (m_pLoopTime)
		delete m_pLoopTime;
	if (m_pLogVariable)
		delete m_pLogVariable;
	if (m_pBlock)
		delete m_pBlock;
}

bool IrisLoopIfStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (!m_pCondition->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (!m_pLoopTime->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (m_pLogVariable && m_pLogVariable->GetType() != IrisIdentifierType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pLogVariable->GetIdentifierString() + " must be a LOCAL VARIABLE name.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::LoopBlock);

		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}

	return true;
}
