#include "IrisComponents/IrisStatements/IrisSetterStatement.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"

IrisSetterStatement::IrisSetterStatement(IrisIdentifier* pSetteredVariable, IrisIdentifier* pParamName, IrisBlock* pBlock) : m_pSetteredVariable(pSetteredVariable), m_pParamName(pParamName), m_pBlock(pBlock)
{
}


bool IrisSetterStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);
	pCompiler->PushUpperType(IrisCompiler::UpperType::Loop);

	if (m_pSetteredVariable->GetType() != IrisIdentifilerType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pSetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.");
		return false;
	}

	const string& strMethodName = m_pSetteredVariable->GetIdentifierString();

	IR_BYTE bWithBlock = m_pBlock ? 1 : 0;

	pCompiler->IncreamDefineIndex();
	pMaker->str_def(pCompiler->GetIdentifierIndex(strMethodName, pCompiler->GetCurrentFileIndex()), pCompiler->GetIdentifierIndex(m_pParamName ? m_pParamName->GetIdentifierString() : "value", pCompiler->GetCurrentFileIndex()), bWithBlock, pCompiler->GetDefineIndex());

	if (m_pBlock) {
		if (!m_pBlock->Generate()) {
			return false;
		}
	}

	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	pCompiler->PopUpperType();

	return true;
}

IrisSetterStatement::~IrisSetterStatement()
{
	if (m_pSetteredVariable) {
		delete m_pSetteredVariable;
	}
	if (m_pParamName) {
		delete m_pParamName;
	}
	if (m_pBlock) {
		delete m_pBlock;
	}
}
