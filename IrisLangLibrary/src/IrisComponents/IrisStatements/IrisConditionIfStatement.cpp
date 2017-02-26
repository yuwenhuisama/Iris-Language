#include "IrisComponents/IrisStatements/IrisConditionIfStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisComponents/IrisParts/IrisElseIf.h"
#include "IrisInstructorMaker.h"
#include "IrisUnil/IrisIdentifier.h"

#include "IrisValidator/IrisExpressionValidateVisitor.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"

#include <vector>
using namespace std;

bool IrisConditionIfStatement::Generate() {
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);


	auto pJumpToEnd = new IrisInstructorMaker::Label();
	auto pJumpSkip = new IrisInstructorMaker::Label();

	// 装入条件
	if (!m_pCondition->Generate()) {
		return false;
	}

	pMaker->jfon(pJumpSkip);

	// 装入Block
	if (!m_pBlock->Generate()) {
		return false;
	}
	pMaker->jmp(pJumpToEnd);
	pMaker->place_lable(pJumpSkip);

	if (m_pIrisElseIf) {
		if(!m_pIrisElseIf->Ergodic(
			[&](IrisElseIf*& pElseIf) -> bool {

			auto pJumpSkip = new IrisInstructorMaker::Label();

			if (!pElseIf->m_pCondition->Generate()) {
				return false;
			}

			pMaker->jfon(pJumpSkip);

			if (!pElseIf->m_pBlock->Generate()) {
				return false;
			}
			pMaker->jmp(pJumpToEnd);
			pMaker->place_lable(pJumpSkip);
			return true;
		}
		))
			return false;
	}
	
	if (m_pElseBlock) {
		if (!m_pElseBlock->Generate()) {
			return false;
		}
	}

	pMaker->place_lable(pJumpToEnd);

	return true;
}

//bool IrisConditionIfStatement::Generate()
//{
//	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
//	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
//	pCompiler->SetLineNumber(m_nLineNumber);
//
//	vector<vector<IR_WORD>*> vcBlocksCodes;
//	vector<IR_WORD>* vcElseBlockCodes = nullptr;
//	vector<vector<IR_WORD>*> vcConditionsCodes;
//
//	vector<IR_WORD>* pNewVector = nullptr;
//
//	vector<IR_WORD> vcNewVector;
//	vector<IR_WORD>* pOldVector = pCompiler->GetCurrentCodeVector();
//	pCompiler->SetCurrentCodeList(&vcNewVector);
//
//	// 装入条件
//	if (!m_pCondition->Generate()) {
//		return false;
//	}
//	pNewVector = new vector<IR_WORD>();
//	pNewVector->assign(vcNewVector.begin(), vcNewVector.end());
//	vcConditionsCodes.push_back(pNewVector);
//
//	if (m_pIrisElseIf) {
//		if(!m_pIrisElseIf->Ergodic(
//			[&](IrisElseIf*& pElseIf) -> bool {
//			vcNewVector.clear();
//			if (!pElseIf->m_pCondition->Generate()) {
//				return false;
//			}
//			pNewVector = new vector<IR_WORD>();
//			pNewVector->assign(vcNewVector.begin(), vcNewVector.end());
//			vcConditionsCodes.push_back(pNewVector);
//			return true;
//		}
//		))
//			return false;
//	}
//	
//	// 装入Block
//	vcNewVector.clear();
//	if (!m_pBlock->Generate()) {
//		return false;
//	}
//	pNewVector = new vector<IR_WORD>();
//	pNewVector->assign(vcNewVector.begin(), vcNewVector.end());
//	vcBlocksCodes.push_back(pNewVector);
//	if (m_pIrisElseIf) {
//		if(!m_pIrisElseIf->Ergodic(
//			[&](IrisElseIf*& pElseIf) -> bool {
//			vcNewVector.clear();
//			if (!pElseIf->m_pBlock->Generate()) {
//				return false;
//			}
//			pNewVector = new vector<IR_WORD>();
//			pNewVector->assign(vcNewVector.begin(), vcNewVector.end());
//			vcBlocksCodes.push_back(pNewVector);
//			return true;
//		}
//		))
//			return false;
//	}
//
//	if (m_pElseBlock) {
//		vcNewVector.clear();
//		if (!m_pElseBlock->Generate()) {
//			return false;
//		}
//		vcElseBlockCodes = new vector <IR_WORD>();
//		vcElseBlockCodes->assign(vcNewVector.begin(), vcNewVector.end());
//	}
//
//	pCompiler->SetCurrentCodeList(pOldVector);
//
//	unsigned int nAllCodeSize = 0;
//	auto iCond = vcConditionsCodes.begin();
//	auto iBlock = vcBlocksCodes.begin();
//	for (; iCond != vcConditionsCodes.end(); ++iCond, ++iBlock) {
//		nAllCodeSize += (*iCond)->size();
//		nAllCodeSize += (*iBlock)->size();
//		nAllCodeSize += 5 + 5; // jnof and jmp
//	}
//
//	if (m_pElseBlock) {
//		nAllCodeSize += vcElseBlockCodes->size();
//	}
//
//	// 开始生成跳转链
//	iCond = vcConditionsCodes.begin();
//	iBlock = vcBlocksCodes.begin();
//	for (; iCond != vcConditionsCodes.end(); ++iCond, ++iBlock) {
//		// Condition
//		pCompiler->LinkCodesToRealCodes(*(*iCond));
//		// if not the last block then it has jmp
//		if (*iCond != vcConditionsCodes.back()) {
//			pMaker->jfon((*iBlock)->size() + 4 + 1);
//		}
//		else {
//			if (m_pElseBlock) {
//				pMaker->jfon((*iBlock)->size() + 4 + 1);
//
//			}
//			else {
//				pMaker->jfon((*iBlock)->size());
//
//			}
//		}
//
//		pCompiler->LinkCodesToRealCodes(*(*iBlock));
//		if (*iCond != vcConditionsCodes.back()) {
//			nAllCodeSize -= ((*iCond)->size() + (*iBlock)->size() + 5 + 5);
//			pMaker->jmp(nAllCodeSize - 5);
//		}
//		else {
//			// last condition and has else
//			if (m_pElseBlock) {
//				nAllCodeSize -= ((*iCond)->size() + (*iBlock)->size() + 5 + 5);
//				pMaker->jmp(nAllCodeSize - 5);
//			}
//		}
//	}
//	// else
//	if (m_pElseBlock) {
//		pCompiler->LinkCodesToRealCodes(*vcElseBlockCodes);
//	}
//
//	// Release
//	for (auto block : vcBlocksCodes) {
//		delete block;
//	}
//	for (auto condition : vcConditionsCodes) {
//		delete condition;
//	}
//	if (vcElseBlockCodes) {
//		delete vcElseBlockCodes;
//	}
//
//	return true;
//}

IrisConditionIfStatement::IrisConditionIfStatement(IrisExpression* pCondition, IrisBlock* pBlock, IrisList<IrisElseIf*>* pIrisElseIf, IrisBlock* pElseBlock) : m_pCondition(pCondition), m_pBlock(pBlock), m_pIrisElseIf(pIrisElseIf), m_pElseBlock(pElseBlock)
{
}


IrisConditionIfStatement::~IrisConditionIfStatement()
{
	if (m_pCondition)
		delete m_pCondition;
	if (m_pBlock)
		delete m_pBlock;
	if (m_pIrisElseIf) {
		m_pIrisElseIf->Ergodic([](IrisElseIf*& x) -> bool { delete x; x = nullptr; return true; });
		m_pIrisElseIf->Clear();
		delete m_pIrisElseIf;
	}
	if (m_pElseBlock)
		delete m_pElseBlock;
}

bool IrisConditionIfStatement::Validate()
{
	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (!m_pCondition->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (m_pBlock && !m_pBlock->Accept(&isvvStatementVisitor)) {
		return false;
	}

	if (m_pIrisElseIf && !m_pIrisElseIf->Ergodic([&](IrisElseIf*& pElseIf) -> bool {

		if (!pElseIf->m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}
		if (!pElseIf->m_pCondition->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pElseBlock && !m_pElseBlock->Accept(&isvvStatementVisitor)) {
		return false;
	}

	return true;
}
