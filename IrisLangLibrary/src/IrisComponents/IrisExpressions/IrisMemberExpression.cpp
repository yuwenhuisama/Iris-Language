#include "IrisComponents/IrisExpressions/IrisMemberExpression.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"

bool IrisMemberExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	if (!m_pCaller->Generate()) {
		return false;
	}
	pMaker->load(IrisAMType::MemberValue, pCompiler->GetIdentifierIndex(m_pProperty->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));

	return true;
}

bool IrisMemberExpression::LeftValue(IrisAMType & eType, IR_DWORD & bIndex)
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker *pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push();
	if (!m_pCaller->Generate()) {
		return false;
	}
	eType = IrisAMType::MemberValue;
	bIndex = pCompiler->GetIdentifierIndex(m_pProperty->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

	return true;
}

IrisMemberExpression::IrisMemberExpression(IrisExpression* pCaller, IrisIdentifier* pPropery) : m_pCaller(pCaller), m_pProperty(pPropery)
{
}


IrisMemberExpression::~IrisMemberExpression()
{
	if (m_pCaller) {
		delete m_pCaller;
	}
	if (m_pProperty) {
		delete m_pProperty;
	}

}
