#include "IrisComponents/IrisExpressions/IrisUnaryExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"


bool IrisUnaryExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (!m_pExpression->Generate()) {
		return false;
	}

	pMaker->push_env();
	pMaker->cre_env();

	switch (m_eType)
	{
	case LogicNot:
		pMaker->nol_call(pCompiler->GetIdentifierIndex("!", pCompiler->GetCurrentFileIndex()), pCompiler->GetCurrentFileIndex());
		break;
	case BitNot:
		pMaker->nol_call(pCompiler->GetIdentifierIndex("~", pCompiler->GetCurrentFileIndex()), pCompiler->GetCurrentFileIndex());
		break;
	case Minus:
		pMaker->nol_call(pCompiler->GetIdentifierIndex("__minus", pCompiler->GetCurrentFileIndex()), pCompiler->GetCurrentFileIndex());
		break;
	case Plus:
		pMaker->nol_call(pCompiler->GetIdentifierIndex("__plus", pCompiler->GetCurrentFileIndex()), pCompiler->GetCurrentFileIndex());
		break;
	default:
		break;
	}

	pMaker->pop_env();

	return true;
}

IrisUnaryExpression::IrisUnaryExpression(IrisUnaryExpressionType eType, IrisExpression * pExpression) : m_eType(eType), m_pExpression(pExpression)
{
}

IrisUnaryExpression::~IrisUnaryExpression()
{
	if (m_pExpression)
		delete m_pExpression;
}

bool IrisUnaryExpression::Validate()
{

	IrisExpressionValidateVisitor ievvExpressionVisitor;

	if (!m_pExpression->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	return true;
}
