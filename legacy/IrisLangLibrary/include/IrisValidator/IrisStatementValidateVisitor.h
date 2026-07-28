#ifndef _H_IRISSTATEMENTVALIDATEVISITOR_
#define _H_IRISSTATEMENTVALIDATEVISITOR_

#include "IrisValidator/IrisAbstractStatementValidateVisitor.h"

class IrisStatementValidateVisitor : public IrisAbstractStatementValidateVisitor
{
public:
	IrisStatementValidateVisitor();
	~IrisStatementValidateVisitor();

	virtual bool Visit(IrisStatement * pStatement) override;
	virtual bool Visit(IrisBlock * pBlock) override;
};

#endif