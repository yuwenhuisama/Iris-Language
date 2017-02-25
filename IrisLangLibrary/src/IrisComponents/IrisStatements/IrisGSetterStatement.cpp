#include "IrisComponents/IrisStatements/IrisGSetterStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisThread/IrisThreadManager.h"

bool IrisGSetterStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	const string& strMethodName = m_pGSetteredVariable->GetIdentifierString();

	pMaker->gstr_def(pCompiler->GetIdentifierIndex(strMethodName, pCompiler->GetCurrentFileIndex()));

	return true;
}

IrisGSetterStatement::IrisGSetterStatement(IrisIdentifier* pGSetteredVariable) : m_pGSetteredVariable(pGSetteredVariable)
{
}


IrisGSetterStatement::~IrisGSetterStatement()
{
	if (m_pGSetteredVariable) {
		delete m_pGSetteredVariable;
	}
}

bool IrisGSetterStatement::Validate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();

	if (m_pGSetteredVariable->GetType() != IrisIdentifierType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pGSetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	if (pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ClassBlock && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::AccessorStatementIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "accessor Statement can only be used in Class or Module body.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	return true;
}
