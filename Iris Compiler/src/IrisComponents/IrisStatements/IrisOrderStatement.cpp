#include "IrisComponents/IrisStatements/IrisOrderStatement.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"


IrisOrderStatement::IrisOrderStatement(IrisBlock* pOrderBlock, IrisIdentifier* pIrregularObject, IrisBlock* pServeBlock, IrisBlock* pIgnoreBlock) : m_pOrderBlock(pOrderBlock), m_pIrregularObject(pIrregularObject), m_pServeBlock(pServeBlock), m_pIgnoreBlock(pIgnoreBlock)
{
}


bool IrisOrderStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	unsigned int nWithIgnoreBlock = m_pIgnoreBlock ? 1 : 0;

	pCompiler->IncreamDefineIndex();
	pMaker->reg_irp(nWithIgnoreBlock, pCompiler->GetDefineIndex());

	if (!m_pOrderBlock->Generate()) {
		return false;
	}

	vector<IR_WORD>* pOldVector = pCompiler->GetCurrentCodeVector();
	vector<IR_WORD> vcNewVector;
	pCompiler->SetCurrentCodeList(&vcNewVector);
	if (!m_pServeBlock->Generate()) {
		return false;
	}

	vector<IR_WORD> lsAssignIrregular;

	pCompiler->SetCurrentCodeList(&lsAssignIrregular);

	if (m_pIrregularObject->GetType() != IrisIdentifilerType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pIrregularObject->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
		return false;
	}

	pMaker->assign_ir(pCompiler->GetIdentifierIndex(m_pIrregularObject->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));

	pCompiler->SetCurrentCodeList(pOldVector);

	vector<IR_WORD>::iterator iCode = vcNewVector.begin();
	for (int i = 0; i < 4 + 1; ++i) {
		++iCode;
	}

	vcNewVector.insert(iCode, lsAssignIrregular.begin(), lsAssignIrregular.end());
	pCompiler->LinkCodesToRealCodes(vcNewVector);

	if (m_pIgnoreBlock) {
		if (!m_pIgnoreBlock->Generate()) {
			return false;
		}
	}

	pMaker->ureg_irp(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();

	return true;
}

IrisOrderStatement::~IrisOrderStatement()
{
	if (m_pOrderBlock)
		delete m_pOrderBlock;
	if (m_pIrregularObject)
		delete m_pIrregularObject;
	if (m_pServeBlock)
		delete m_pServeBlock;
	if (m_pIgnoreBlock)
		delete m_pIgnoreBlock;
}
