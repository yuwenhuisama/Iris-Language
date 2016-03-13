#include "IrisComponents/IrisStatements/IrisBreakStatement.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"

IrisBreakStatement::IrisBreakStatement(IrisIdentifier* pLabel) : m_pLabel(pLabel)
{
}


bool IrisBreakStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (!pCompiler->UpperWithLoop()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::ContinueIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Continue Statement CAN ONLY be used in loop if statement/for statement.");
		return false;
	}

	pMaker->brk();

	return true;
}

IrisBreakStatement::~IrisBreakStatement()
{
	if (m_pLabel)
		delete m_pLabel;
}
