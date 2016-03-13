#include "IrisComponents/IrisStatements/IrisGetterStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"


bool IrisGetterStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (m_pGetteredVariable->GetType() != IrisIdentifilerType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pGetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.");
		return false;
	}

	const string& strMethodName = m_pGetteredVariable->GetIdentifierString();

	IR_BYTE bWithBlock = m_pBlock ? 1 : 0;

	pCompiler->IncreamDefineIndex();
	pMaker->gtr_def(pCompiler->GetIdentifierIndex(strMethodName, pCompiler->GetCurrentFileIndex()), bWithBlock, pCompiler->GetDefineIndex());

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::Method);
		if (!m_pBlock->Generate()) {
			return false;
		}
		pCompiler->PopUpperType();
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	return true;
}

IrisGetterStatement::IrisGetterStatement(IrisIdentifier* pGetteredVariable, IrisBlock* pBlock) : m_pGetteredVariable(pGetteredVariable), m_pBlock(pBlock)
{
}


IrisGetterStatement::~IrisGetterStatement()
{
	if (m_pGetteredVariable) {
		delete m_pGetteredVariable;
	}
	if (m_pBlock) {
		delete m_pBlock;
	}
}
