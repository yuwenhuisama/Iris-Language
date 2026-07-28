#include "IrisComponents/IrisStatements/IrisBlockStatement.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"
#include "IrisThread/IrisThreadManager.h"


bool IrisBlockStatement::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->blk();
	return true;
}

IrisBlockStatement::IrisBlockStatement()
{
}


IrisBlockStatement::~IrisBlockStatement()
{
}

bool IrisBlockStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	if (!pCompiler->UpperWithBlock()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::BlockStatementIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "block Statement must be used in Closure Block or Method Block.", IrisThreadManager::CurrentThreadManager()->GetMainThreadInfo());
		return false;
	}

	return true;
}
