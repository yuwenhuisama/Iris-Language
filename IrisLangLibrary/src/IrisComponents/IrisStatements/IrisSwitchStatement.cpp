#include "IrisComponents/IrisStatements/IrisSwitchStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisComponents/IrisParts/IrisSwitchBlock.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisComponents/IrisParts/IrisWhen.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"

#include <vector>
using namespace std;

struct  IrisSwitchStatement::_WhenStructure {
	vector<vector<IR_WORD>*> m_lsComparerCodes;
	vector<IR_WORD> m_lsBlockCodes;
	unsigned int m_nComparerSize = 0;

	_WhenStructure() = default;
	~_WhenStructure() {
		for (auto vector : m_lsComparerCodes) {
			delete vector;
		}
	}
};

bool IrisSwitchStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	// maker
	m_pCondition->Generate();
	pMaker->assign_cmp();

	vector<IR_WORD>* pOldVector = pCompiler->GetCurrentCodeVector();
	vector<IR_WORD>* pNewVector = nullptr;
	vector<IR_WORD>* pElseVector = nullptr;
	_WhenStructure* pWhenStructure = nullptr;
	vector<_WhenStructure*> vcWhenStructures;
	size_t nWholeSize = 0;

	// Generate
	for(auto when : m_pSwitchBlock->m_pWhenList->m_lsList){
		pWhenStructure = new _WhenStructure();

		if(!when->m_pExpressions->Ergodic(
			[&](IrisExpression* pExpression) -> bool {
			pNewVector = new vector<IR_WORD>();
			pCompiler->SetCurrentCodeList(pNewVector);
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->push_env();
			pMaker->cre_env();
			pMaker->cmp_cmp();
			pMaker->pop_env();
			pWhenStructure->m_lsComparerCodes.push_back(pNewVector);
			pWhenStructure->m_nComparerSize += pNewVector->size() + 4 + 1; // compare size + jfon or jt
			return true;
		}
			)) {
			return false;
		}

		pCompiler->SetCurrentCodeList(&(pWhenStructure->m_lsBlockCodes));
		//pNewVector = new vector<IR_WORD>();
		if (!when->m_pBlock->Generate()) {
			return false;
		}

		vcWhenStructures.push_back(pWhenStructure);
		nWholeSize += pWhenStructure->m_lsBlockCodes.size() + pWhenStructure->m_nComparerSize + 5; // + jmp
	}

	if (m_pSwitchBlock->m_pElseBlock) {
		pElseVector = new vector<IR_WORD>();
		pCompiler->SetCurrentCodeList(pElseVector);
		m_pSwitchBlock->m_pElseBlock->Generate();
		nWholeSize += pElseVector->size();
	}

	pCompiler->SetCurrentCodeList(pOldVector);

	// Calc
	unsigned int Offset = 0;
	for (auto when : vcWhenStructures) {
		Offset = when->m_nComparerSize;
		for (auto cmp : when->m_lsComparerCodes) {
			pCompiler->LinkCodesToRealCodes(*cmp);
			if (cmp != when->m_lsComparerCodes.back()) {
				pMaker->jt(Offset - cmp->size() + 5); // - jmp
			}
			else {
				if (when != vcWhenStructures.back()) {
					pMaker->jfon(when->m_lsBlockCodes.size() + 4 + 1); // skip the block and jmp
				}
				else {
					if(!pElseVector) {
						pMaker->jfon(when->m_lsBlockCodes.size()); // skip the block
					}
					else {
						pMaker->jfon(when->m_lsBlockCodes.size() + 5); // skip the block
					}
				}
			}
		}
		pCompiler->LinkCodesToRealCodes(when->m_lsBlockCodes);
		nWholeSize -= when->m_lsBlockCodes.size() + when->m_nComparerSize + 5;
		if(nWholeSize != 0) {
			pMaker->jmp(nWholeSize - 5);
		}
	}

	if (pElseVector) {
		pCompiler->LinkCodesToRealCodes(*pElseVector);
	}

	for (auto strt : vcWhenStructures) {
		delete strt;
	}
	if(pElseVector) {
		delete pElseVector;
	}

	return true;
}

IrisSwitchStatement::IrisSwitchStatement(IrisExpression* pCondition, IrisSwitchBlock* pSwitchBlock) : m_pCondition(pCondition), m_pSwitchBlock(pSwitchBlock)
{
}


IrisSwitchStatement::~IrisSwitchStatement()
{
	if (m_pCondition)
		delete m_pCondition;
	if (m_pSwitchBlock)
		delete m_pSwitchBlock;
}

bool IrisSwitchStatement::Validate()
{
	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (!m_pCondition->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (m_pSwitchBlock) {
		if (!m_pSwitchBlock->m_pWhenList->Ergodic([&](IrisWhen*& pWhen) -> bool {
			if (!pWhen->m_pExpressions->Ergodic([&](IrisExpression*& pExpression) -> bool {
				if (!pExpression->Accept(&ievvExpressionVisitor)) {
					return false;
				}
				return true;
			})) {
				return false;
			}

			if (!pWhen->m_pBlock->Accept(&isvvStatementVisitor)) {
				return false;
			}
			return true;
		})) {
			return false;
		}

		if (!m_pSwitchBlock->m_pElseBlock && !m_pSwitchBlock->m_pElseBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}
	}

	return true;
}
