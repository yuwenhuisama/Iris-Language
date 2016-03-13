#include "IrisComponents/IrisStatements/IrisGSetterStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"


bool IrisGSetterStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (m_pGSetteredVariable->GetType() != IrisIdentifilerType::InstanceVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pGSetteredVariable->GetIdentifierString() + " is not a INSTANCE VARIABLE.");
		return false;
	}

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
