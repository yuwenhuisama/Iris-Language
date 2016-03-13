#include "IrisComponents/IrisStatements/IrisAuthorityStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"

IrisAuthorityStatement::IrisAuthorityStatement(IrisAuthorityEnvironment eEnv, IrisAuthorityTarget eTar, IrisAuthorityType eType, IrisIdentifier* pMethodName) : m_eEnvironment(eEnv), m_eTarget(eTar), m_eType(eType), m_pMethodName(pMethodName)
{
}


bool IrisAuthorityStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);
	
	if (m_pMethodName->GetType() != IrisIdentifilerType::LocalVariable) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Only method can be given authirity.");
		return false;
	}

	pMaker->set_auth(pCompiler->GetIdentifierIndex(m_pMethodName->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), (IR_BYTE)m_eEnvironment, (IR_BYTE)m_eType, (IR_BYTE)m_eType);

	return true;
}

IrisAuthorityStatement::~IrisAuthorityStatement()
{
	if (m_pMethodName) {
		delete m_pMethodName;
	}
}
