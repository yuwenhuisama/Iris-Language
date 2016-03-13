#include "IrisComponents/IrisStatements/IrisBlockStatement.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"


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
