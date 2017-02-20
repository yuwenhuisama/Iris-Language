#include "IrisComponents/IrisStatements/IrisAuthorityStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"

IrisAuthorityStatement::IrisAuthorityStatement(IrisAuthorityTarget eTar, IrisAuthorityType eType, IrisIdentifier* pMethodName) : m_eTarget(eTar), m_eType(eType), m_pMethodName(pMethodName)
{
}

bool IrisAuthorityStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->set_auth(pCompiler->GetIdentifierIndex(m_pMethodName->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), (IR_BYTE)m_eEnvironment, (IR_BYTE)m_eTarget, (IR_BYTE)m_eType);

	return true;
}

void IrisAuthorityStatement::SetAuthorityEnvironment(IrisAuthorityEnvironment eEnv)
{
	m_eEnvironment = eEnv;
}

IrisAuthorityStatement::~IrisAuthorityStatement()
{
	if (m_pMethodName) {
		delete m_pMethodName;
	}
}

bool IrisAuthorityStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	
	if (pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ClassBlock
		&& pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::AuthorityStatementIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "authority Statement can only be used in Class or Module body.");
		return false;
	}

	if (m_pMethodName->GetType() != IrisIdentifierType::LocalVariable && m_pMethodName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pMethodName->GetIdentifierString() + " must be a local variable name or a constance variable name.");
		return false;
	}

	return true;
}
