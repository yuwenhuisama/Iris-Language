#include "IrisComponents/IrisStatements/IrisSetterStatement.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"


IrisSetterStatement::IrisSetterStatement(IrisIdentifier* pSetteredVariable, IrisIdentifier* pParamName, IrisBlock* pBlock) : m_pSetteredVariable(pSetteredVariable), m_pParamName(pParamName), m_pBlock(pBlock)
{
}


bool IrisSetterStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

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

bool IrisSetterStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ClassBlock && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::AccessorStatementIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "accessor Statement can only be used in Class or Module body.");
		return false;
	}

	if (m_pSetteredVariable->GetType() != IrisIdentifierType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pSetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.");
		return false;
	}

	if (m_pParamName->GetType() != IrisIdentifierType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pParamName->GetIdentifierString() + " must be a LOCAL VARIABLE name.");
		return false;
	}

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::MethodBlock);

		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}

	return true;
}
