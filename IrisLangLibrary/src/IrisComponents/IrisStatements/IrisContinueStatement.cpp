#include "IrisComponents/IrisStatements/IrisContinueStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"
#include "IrisThread/IrisThreadManager.h"

bool IrisContinueStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	// fake offset
	pMaker->ctn(static_cast<IrisInstructorMaker::Label*>(m_pCurrentLoopEndLabel));

	return true;
}

IrisContinueStatement::IrisContinueStatement(IrisIdentifier* pLabel) : m_pLabel(pLabel)
{
}


IrisContinueStatement::~IrisContinueStatement()
{
	if (m_pLabel)
		delete m_pLabel;
}

bool IrisContinueStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	if (!pCompiler->UpperWithLoop()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ContinueIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "continue Statement CAN ONLY be used in loop if statement/for statement.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	return true;
}
